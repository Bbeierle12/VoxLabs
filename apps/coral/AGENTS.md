# AutoResearch Integration Protocol for AI Agents

Welcome, AI Agent! If you have been assigned to optimize, tune, or rewrite code in this repository, you are participating in the **AutoResearch Loop**. 

The user is monitoring your progress via a live desktop dashboard. To ensure your progress is accurately reflected on the user's screen in real-time, you **MUST** adhere to the following data deposition protocols.

## 1. The Dashboard Architecture (Push via Flat-File)
You do not need to send HTTP requests to update the dashboard. The dashboard acts as a reactive viewer that polls specific flat-files on the disk every 3 seconds. 
**Your job is simply to append to these files in the exact formats specified below.** The dashboard will automatically detect the changes and update the UI.

## 2. Recording Metric Runs (`results.tsv`)
Every time you run an evaluation or benchmark, you must log the result by appending a single row to `results.tsv` in the root of the project.

**Rules for `results.tsv`:**
- It is a Tab-Separated Values (TSV) file.
- Do NOT modify the header row.
- **Row Format:** 
  `[ISO 8601 Timestamp] \t [Commit Hash OR "revert"] \t [Primary Metric] \t [Secondary Metrics...] \t [Hyperparameters...] \t [Rationale/Comment]`
- **Reverts:** If your hypothesis failed to improve the primary metric over the historical baseline, you must revert the code changes and log the run with `"revert"` in the commit column. The dashboard explicitly tracks reverts to calculate success rates.
- **Commits:** If your hypothesis improved the metric, leave the code modified, and log a unique ID (or git commit hash) in the commit column.

## 3. Reporting Algorithmic Bottlenecks (`discoveries.log`)
If you hit a local maximum, discover a structural flaw in the code, or realize that hyperparameter tuning is no longer sufficient, you must log this issue so the user (or a heavier cloud agent) can step in and rewrite the architecture.

**Rules for `discoveries.log`:**
- Append your findings to `discoveries.log` in the root of the project.
- Use the following format separated by double newlines:
```
[YYYY-MM-DD HH:MM:SS UTC] Rationale: <Brief explanation of what you were trying>
Issue Discovered: <Detailed explanation of the architectural bottleneck or mathematical limitation>

```
- The dashboard parses this file and displays it in the "Algorithmic Discoveries & Bottlenecks" UI panel.

## 4. Execution Permissions
If you are a cloud-based agent executing commands autonomously, ensure you have requested the necessary persistent permissions (e.g., `ask_permission` for `npm run bench`) so you do not get stuck waiting for user approval prompts.

By following this file-based contract, you ensure the human supervisor has perfect real-time visibility into your autonomous research loop.
