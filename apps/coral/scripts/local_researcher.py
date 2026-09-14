import os
import sys
import json
import time
import subprocess
import re
from datetime import datetime
from urllib import request, error

# Configuration
WORKSPACE_DIR = "/home/bbeierle12/Coral"
RESULTS_FILE = os.path.join(WORKSPACE_DIR, "results.tsv")
DISCOVERIES_FILE = os.path.join(WORKSPACE_DIR, "discoveries.log")
TARGET_FILE = os.path.join(WORKSPACE_DIR, "src/config/detector.ts")
OLLAMA_URL = "http://localhost:11434/api/generate"
MODEL_NAME = "gemma4:latest"

def read_file(path):
    with open(path, "r", encoding="utf-8") as f:
        return f.read()

def write_file(path, content):
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

def read_history():
    if not os.path.exists(RESULTS_FILE):
        return []
    
    with open(RESULTS_FILE, "r", encoding="utf-8") as f:
        lines = f.readlines()
        
    if len(lines) <= 1:
        return []
        
    headers = lines[0].strip().split("\t")
    history = []
    for line in lines[1:]:
        if not line.strip(): continue
        parts = line.strip().split("\t")
        row = dict(zip(headers, parts))
        history.append(row)
    return history

def get_best_f1(history):
    best_f1 = 0.0
    for row in history:
        if row.get("commit") != "revert":
            try:
                f1 = float(row.get("f1", 0.0))
                if f1 > best_f1:
                    best_f1 = f1
            except:
                pass
    return best_f1

def generate_hypothesis(history, current_code):
    prompt = f"""You are an expert autonomous researcher tuning a Choral Note Detector. 
Your goal is to maximize the F1 score by tweaking the numeric hyperparameters in the detector config.

Here is the current configuration code:
```typescript
{current_code}
```

Here is the recent history of experiments (the last column is the comment/rationale):
{json.dumps(history[-10:], indent=2)}

Think step-by-step about what parameter changes might improve the F1 score based on the historical results.
Also, analyze if there are any major architectural bottlenecks or issues that parameter tuning can't fix.

You MUST respond with a single valid JSON object in the following format. Do not output any markdown blocks or extra text outside the JSON.
{{
  "parameters": {{
    "harmonicCount": 9,
    "salienceThreshold": 3.0,
    "whiteningWindowHz": 2000,
    "maxPolyphony": 8,
    "cancelFactor": 0.9,
    "mergeRadius": 1,
    "decayPerFrame": 0.92
  }},
  "rationale": "A brief explanation of why you made these changes.",
  "issue": "A brief explanation of any major algorithmic limitations you've hit (or null if none)."
}}
"""
    data = {
        "model": MODEL_NAME,
        "prompt": prompt,
        "stream": False,
        "format": "json"
    }
    
    req = request.Request(OLLAMA_URL, data=json.dumps(data).encode("utf-8"), headers={'Content-Type': 'application/json'})
    try:
        response = request.urlopen(req)
        result = json.loads(response.read().decode("utf-8"))
        return json.loads(result["response"])
    except Exception as e:
        print(f"Error querying Ollama: {e}")
        return None

def apply_parameters(code, params):
    modified = code
    for k, v in params.items():
        # Match parameter keys like "salienceThreshold: 3.0,"
        pattern = rf"(\b{re.escape(k)}\b\s*:\s*)([\d.-]+|true|false)(,?)"
        val_str = str(v).lower() if isinstance(v, bool) else str(v)
        modified = re.sub(pattern, rf"\g<1>{val_str}\g<3>", modified)
    return modified

def log_issue(issue, rationale):
    if not issue: return
    timestamp = datetime.utcnow().strftime("%Y-%m-%d %H:%M:%S UTC")
    log_entry = f"[{timestamp}] Rationale: {rationale}\nIssue Discovered: {issue}\n\n"
    with open(DISCOVERIES_FILE, "a", encoding="utf-8") as f:
        f.write(log_entry)
    print(f"Logged new issue to discoveries.log: {issue}")

def run_benchmark():
    try:
        result = subprocess.run(
            ["npm", "run", "bench"],
            cwd=WORKSPACE_DIR,
            capture_output=True,
            text=True,
            check=False
        )
        
        output = result.stdout
        
        # Parse metrics
        if "METRICS_JSON_START" in output and "METRICS_JSON_END" in output:
            start_idx = output.find("METRICS_JSON_START") + len("METRICS_JSON_START")
            end_idx = output.find("METRICS_JSON_END")
            json_str = output[start_idx:end_idx].strip()
            metrics = json.loads(json_str)
            return metrics
        else:
            print("Failed to find METRICS_JSON in output.")
            return None
    except Exception as e:
        print(f"Failed to run benchmark: {e}")
        return None

def main():
    print("Starting local Gemma4 autonomous research loop...")
    
    while True:
        print("\n--- Starting New Iteration ---")
        history = read_history()
        best_f1 = get_best_f1(history)
        print(f"Current Best F1: {best_f1:.4f}")
        
        current_code = read_file(TARGET_FILE)
        
        print("Querying Gemma4 for next hypothesis...")
        hypothesis = generate_hypothesis(history, current_code)
        
        if not hypothesis or "parameters" not in hypothesis:
            print("Failed to get valid hypothesis. Retrying in 5 seconds...")
            time.sleep(5)
            continue
            
        params = hypothesis["parameters"]
        rationale = hypothesis.get("rationale", "Auto-tuned parameters")
        issue = hypothesis.get("issue", None)
        
        print(f"Hypothesis generated:\n{json.dumps(params, indent=2)}\nRationale: {rationale}")
        
        # Log issue if found
        if issue:
            log_issue(issue, rationale)
            
        # Apply parameters
        new_code = apply_parameters(current_code, params)
        if new_code == current_code:
            print("No parameters were changed by the regex. Retrying...")
            time.sleep(5)
            continue
            
        write_file(TARGET_FILE, new_code)
        
        print("Running benchmark...")
        metrics = run_benchmark()
        
        if not metrics or "overall" not in metrics:
            print("Benchmark failed or returned invalid data. Reverting changes...")
            write_file(TARGET_FILE, current_code)
            time.sleep(5)
            continue
            
        f1 = metrics["overall"].get("f1", 0.0)
        precision = metrics["overall"].get("precision", 0.0)
        recall = metrics["overall"].get("recall", 0.0)
        
        print(f"Benchmark finished. New F1: {f1:.4f} (Baseline: {best_f1:.4f})")
        
        timestamp = datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
        
        # We assume the results.tsv has specific columns. We construct the row dynamically based on the first line.
        with open(RESULTS_FILE, "r") as f:
            headers = f.readline().strip().split("\t")
            
        row_data = {
            "timestamp": timestamp,
            "f1": f"{f1:.6f}",
            "precision": f"{precision:.6f}",
            "recall": f"{recall:.6f}",
            "byTag": json.dumps(metrics.get("byTag", {})),
            "comment": rationale
        }
        
        for k, v in params.items():
            row_data[k] = str(v)
            
        if f1 > best_f1:
            print(f"🎉 New best F1 score! Committing changes.")
            row_data["commit"] = "gemma-" + str(int(time.time()))[-6:]
            # We leave TARGET_FILE modified (committed)
        else:
            print(f"❌ Score did not improve. Reverting changes.")
            row_data["commit"] = "revert"
            write_file(TARGET_FILE, current_code) # Revert
            
        # Construct line
        line = []
        for h in headers:
            line.append(row_data.get(h, ""))
            
        with open(RESULTS_FILE, "a") as f:
            f.write("\t".join(line) + "\n")
            
        print("Iteration complete. Waiting 2 seconds before next loop...")
        time.sleep(2)

if __name__ == "__main__":
    main()
