package org.vocaltract.pixel;

import android.app.Activity;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.graphics.Color;
import android.graphics.Insets;
import android.media.AudioAttributes;
import android.media.AudioFocusRequest;
import android.media.AudioManager;
import android.os.Build;
import android.os.Bundle;
import android.os.SystemClock;
import android.view.View;
import android.view.ViewGroup;
import android.view.ViewParent;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.webkit.JavascriptInterface;
import android.webkit.PermissionRequest;
import android.webkit.WebChromeClient;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import java.util.List;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.RejectedExecutionException;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;
import kotlin.Metadata;
import kotlin.Pair;
import kotlin.Result;
import kotlin.ResultKt;
import kotlin.Triple;
import kotlin.TuplesKt;
import kotlin.Unit;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.collections.MapsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.functions.Function2;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;
import org.json.JSONArray;
import org.json.JSONObject;
import org.vocaltract.pixel.AdditiveSynthState;
import org.vocaltract.pixel.AnatomyBenchActivity;

/* compiled from: AnatomyBenchActivity.kt */
@Metadata(d1 = {"\u0000¸\u0001\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0014\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0004\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\t\n\u0002\b\u0006\n\u0002\u0010\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\b\u0007\n\u0002\u0010\u0011\n\u0000\n\u0002\u0010\u0015\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\b\u0018\u00002\u00020\u0001:\u0001SB\u0007¢\u0006\u0004\b\u0002\u0010\u0003J\u0012\u00100\u001a\u0002012\b\u00102\u001a\u0004\u0018\u000103H\u0014J\b\u00104\u001a\u000201H\u0002J\u0010\u00105\u001a\u0002012\u0006\u00106\u001a\u000207H\u0002J\u0010\u00108\u001a\u0002012\u0006\u00109\u001a\u00020:H\u0002J7\u0010;\u001a\u0002012\u000e\b\u0002\u0010<\u001a\b\u0012\u0004\u0012\u0002010=2\n\b\u0002\u0010>\u001a\u0004\u0018\u00010:2\f\u0010?\u001a\b\u0012\u0004\u0012\u0002010=H\u0002¢\u0006\u0002\u0010@J\b\u0010A\u001a\u000201H\u0002J-\u0010B\u001a\u0002012\u0006\u0010C\u001a\u00020:2\u000e\u0010D\u001a\n\u0012\u0006\b\u0001\u0012\u00020(0E2\u0006\u0010F\u001a\u00020GH\u0016¢\u0006\u0002\u0010HJ\u0010\u0010I\u001a\u0002012\u0006\u0010J\u001a\u00020KH\u0002J\b\u0010L\u001a\u000201H\u0002J\b\u0010M\u001a\u000201H\u0002J\b\u0010N\u001a\u000201H\u0002J\b\u0010O\u001a\u000201H\u0002J\b\u0010P\u001a\u000201H\u0014J\b\u0010Q\u001a\u000201H\u0014J\b\u0010R\u001a\u000201H\u0014R\u000e\u0010\u0004\u001a\u00020\u0005X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0007X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\b\u001a\u00020\tX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\u000bX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\f\u001a\u00020\rX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u000e\u001a\u00020\u000fX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0010\u001a\u00020\u0011X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0012\u001a\u00020\u0013X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0014\u001a\u00020\u0015X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0016\u001a\u00020\u0017X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0018\u001a\u00020\u0019X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001a\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001c\u001a\u00020\u001dX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u001e\u001a\u00020\u001fX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010 \u001a\u00020!X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\"\u001a\u00020#X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010$\u001a\u00020#X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010%\u001a\u00020#X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010&\u001a\u00020#X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010'\u001a\u00020(X\u0082\u000e¢\u0006\u0002\n\u0000R\u0012\u0010)\u001a\u0004\u0018\u00010*X\u0082\u000e¢\u0006\u0004\n\u0002\u0010+R\u000e\u0010,\u001a\u00020(X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010-\u001a\u00020(X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010.\u001a\u00020*X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010/\u001a\u0004\u0018\u00010(X\u0082\u000e¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/AnatomyBenchActivity;", "Landroid/app/Activity;", "<init>", "()V", "web", "Landroid/webkit/WebView;", "model", "Lorg/vocaltract/pixel/SharedTractModel;", "state", "Lorg/vocaltract/pixel/SharedLabState;", "synth", "Lorg/vocaltract/pixel/AdditiveSynthEngine;", "pipeline", "Lorg/vocaltract/pixel/RealtimeDspPipeline;", "capture", "Lorg/vocaltract/pixel/AudioCaptureEngine;", "replay", "Lorg/vocaltract/pixel/ReplayController;", "calibration", "Lorg/vocaltract/pixel/SingerCalibrationController;", "audioManager", "Landroid/media/AudioManager;", "focusRequest", "Landroid/media/AudioFocusRequest;", "currentSound", "Lorg/vocaltract/pixel/AdditiveSynthState;", "currentCoordinates", "", "executor", "Ljava/util/concurrent/ThreadPoolExecutor;", "epoch", "Ljava/util/concurrent/atomic/AtomicLong;", "livePending", "Ljava/util/concurrent/atomic/AtomicBoolean;", "resumed", "", "trusted", "destroyed", "playWanted", "inputMode", "", "permissionEpoch", "", "Ljava/lang/Long;", "calibrationText", "statusMessage", "lastUiFrame", "latestSnapshot", "onCreate", "", "savedInstanceState", "Landroid/os/Bundle;", "buildUi", "handle", "c", "Lorg/vocaltract/pixel/LabCommand;", "acknowledge", "id", "", "submit", "onDone", "Lkotlin/Function0;", "requestId", "change", "(Lkotlin/jvm/functions/Function0;Ljava/lang/Integer;Lkotlin/jvm/functions/Function0;)V", "beginCapture", "onRequestPermissionsResult", "requestCode", "permissions", "", "grantResults", "", "(I[Ljava/lang/String;[I)V", "acceptObservation", "observation", "Lorg/vocaltract/pixel/VocalAcousticsState;", "stopTone", "stopInput", "stopEverything", "sendStatus", "onPause", "onResume", "onDestroy", "Commands"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AnatomyBenchActivity extends Activity {
    private AudioManager audioManager;
    private SingerCalibrationController calibration;
    private AudioCaptureEngine capture;
    private volatile boolean destroyed;
    private AudioFocusRequest focusRequest;
    private long lastUiFrame;
    private String latestSnapshot;
    private SharedTractModel model;
    private Long permissionEpoch;
    private RealtimeDspPipeline pipeline;
    private boolean playWanted;
    private ReplayController replay;
    private volatile boolean resumed;
    private SharedLabState state;
    private AdditiveSynthEngine synth;
    private volatile boolean trusted;
    private WebView web;
    private AdditiveSynthState currentSound = AdditiveSynthState.Companion.create$default(AdditiveSynthState.INSTANCE, 0.0f, 0.0f, null, false, 15, null);
    private float[] currentCoordinates = new float[4];
    private final ThreadPoolExecutor executor = new ThreadPoolExecutor(1, 1, 0, TimeUnit.MILLISECONDS, new ArrayBlockingQueue(32));
    private final AtomicLong epoch = new AtomicLong(0);
    private final AtomicBoolean livePending = new AtomicBoolean(false);
    private String inputMode = "idle";
    private String calibrationText = "Calibration records derived summaries only.";
    private String statusMessage = "Ready · start at low phone volume.";

    /* compiled from: AnatomyBenchActivity.kt */
    @Metadata(d1 = {"\u0000\u0016\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u0002\n\u0000\n\u0002\u0010\u000e\b\u0086\u0004\u0018\u00002\u00020\u0001B\u0007¢\u0006\u0004\b\u0002\u0010\u0003J\u0010\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u0007H\u0007"}, d2 = {"Lorg/vocaltract/pixel/AnatomyBenchActivity$Commands;", "", "<init>", "(Lorg/vocaltract/pixel/AnatomyBenchActivity;)V", "command", "", "raw", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
    public final class Commands {
        /* JADX DEBUG: Incorrect args count in method signature: ()V */
        public Commands() {
        }

        @JavascriptInterface
        public final void command(String raw) {
            Intrinsics.checkNotNullParameter(raw, "raw");
            try {
                final LabCommand parse = LabCommand.INSTANCE.parse(raw);
                final AnatomyBenchActivity anatomyBenchActivity = AnatomyBenchActivity.this;
                anatomyBenchActivity.runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$Commands$$ExternalSyntheticLambda0
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // java.lang.Runnable
                    public final void run() {
                        AnatomyBenchActivity.Commands.command$lambda$0(AnatomyBenchActivity.this, parse);
                    }
                });
            } catch (Exception unused) {
            }
        }

        /* JADX INFO: Access modifiers changed from: private */
        public static final void command$lambda$0(AnatomyBenchActivity anatomyBenchActivity, LabCommand labCommand) {
            if (anatomyBenchActivity.resumed && anatomyBenchActivity.trusted && !anatomyBenchActivity.destroyed) {
                anatomyBenchActivity.handle(labCommand);
            }
        }
    }

    @Override // android.app.Activity
    protected void onCreate(Bundle savedInstanceState) {
        SharedTractModel sharedTractModel;
        super.onCreate(savedInstanceState);
        DiagnosticsRuntime diagnosticsRuntime = DiagnosticsRuntime.INSTANCE;
        Context applicationContext = getApplicationContext();
        Intrinsics.checkNotNullExpressionValue(applicationContext, "getApplicationContext(...)");
        diagnosticsRuntime.initialize(applicationContext);
        try {
            this.model = new SharedTractModel(ReducedModelAsset.INSTANCE.load(this), TractLumenAsset.INSTANCE.load(this));
            SharedTractModel sharedTractModel2 = this.model;
            SharedTractModel sharedTractModel3 = null;
            if (sharedTractModel2 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel2 = null;
            }
            this.state = new SharedLabState(sharedTractModel2);
            SharedPreferences sharedPreferences = getSharedPreferences("shared_mri_lab", 0);
            SharedLabJson sharedLabJson = SharedLabJson.INSTANCE;
            SharedLabState sharedLabState = this.state;
            if (sharedLabState == null) {
                Intrinsics.throwUninitializedPropertyAccessException("state");
                sharedLabState = null;
            }
            String str = "";
            String string = sharedPreferences.getString("state", "");
            if (string == null) {
                string = "";
            }
            if (!sharedLabJson.restore(sharedLabState, string)) {
                try {
                    Result.Companion companion = Result.INSTANCE;
                    AnatomyBenchActivity anatomyBenchActivity = this;
                    String string2 = getSharedPreferences("combined_lab", 0).getString("state", "");
                    if (string2 != null) {
                        str = string2;
                    }
                    JSONObject jSONObject = new JSONObject(str);
                    SharedLabState sharedLabState2 = this.state;
                    if (sharedLabState2 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                        sharedLabState2 = null;
                    }
                    sharedLabState2.setPitch((float) jSONObject.getDouble("f0"));
                    SharedLabState sharedLabState3 = this.state;
                    if (sharedLabState3 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                        sharedLabState3 = null;
                    }
                    sharedLabState3.setMaster((float) jSONObject.getDouble("master"));
                    this.statusMessage = "Pitch and volume restored. Geometry now uses the shared MRI mean.";
                    Result.m4constructorimpl(Unit.INSTANCE);
                } catch (Throwable th) {
                    Result.Companion companion2 = Result.INSTANCE;
                    Result.m4constructorimpl(ResultKt.createFailure(th));
                }
            }
            SharedLabState sharedLabState4 = this.state;
            if (sharedLabState4 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("state");
                sharedLabState4 = null;
            }
            this.currentSound = sharedLabState4.sound();
            SharedLabState sharedLabState5 = this.state;
            if (sharedLabState5 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("state");
                sharedLabState5 = null;
            }
            float[] coefficients = sharedLabState5.getCoefficients();
            float[] copyOf = Arrays.copyOf(coefficients, coefficients.length);
            Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
            this.currentCoordinates = copyOf;
            Object systemService = getSystemService("audio");
            Intrinsics.checkNotNull(systemService, "null cannot be cast to non-null type android.media.AudioManager");
            this.audioManager = (AudioManager) systemService;
            this.focusRequest = new AudioFocusRequest.Builder(1).setAudioAttributes(new AudioAttributes.Builder().setUsage(1).setContentType(2).build()).setOnAudioFocusChangeListener(new AudioManager.OnAudioFocusChangeListener() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda17
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // android.media.AudioManager.OnAudioFocusChangeListener
                public final void onAudioFocusChange(int i) {
                    AnatomyBenchActivity.onCreate$lambda$3(AnatomyBenchActivity.this, i);
                }
            }).build();
            this.synth = new AdditiveSynthEngine(0, 0, null, null, new Function1() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda18
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function1
                public final Object invoke(Object obj) {
                    Unit onCreate$lambda$5;
                    onCreate$lambda$5 = AnatomyBenchActivity.onCreate$lambda$5(AnatomyBenchActivity.this, (String) obj);
                    return onCreate$lambda$5;
                }
            }, 15, null);
            SharedTractModel sharedTractModel4 = this.model;
            if (sharedTractModel4 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel4 = null;
            }
            ReducedModelAsset atlas = sharedTractModel4.getAtlas();
            SharedTractModel sharedTractModel5 = this.model;
            if (sharedTractModel5 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel = null;
            } else {
                sharedTractModel = sharedTractModel5;
            }
            this.pipeline = new RealtimeDspPipeline(atlas, 0, 0, sharedTractModel, new Function2() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda19
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function2
                public final Object invoke(Object obj, Object obj2) {
                    Unit onCreate$lambda$7;
                    onCreate$lambda$7 = AnatomyBenchActivity.onCreate$lambda$7(AnatomyBenchActivity.this, (VocalAcousticsState) obj, ((Long) obj2).longValue());
                    return onCreate$lambda$7;
                }
            }, new Function1() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda20
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function1
                public final Object invoke(Object obj) {
                    Unit onCreate$lambda$9;
                    onCreate$lambda$9 = AnatomyBenchActivity.onCreate$lambda$9(AnatomyBenchActivity.this, (String) obj);
                    return onCreate$lambda$9;
                }
            }, 6, null);
            Context applicationContext2 = getApplicationContext();
            Intrinsics.checkNotNullExpressionValue(applicationContext2, "getApplicationContext(...)");
            SharedTractModel sharedTractModel6 = this.model;
            if (sharedTractModel6 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel6 = null;
            }
            int sampleRateHz = sharedTractModel6.getAtlas().getSampleRateHz();
            SharedTractModel sharedTractModel7 = this.model;
            if (sharedTractModel7 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel7 = null;
            }
            int frameSize = sharedTractModel7.getAtlas().getFrameSize();
            SharedTractModel sharedTractModel8 = this.model;
            if (sharedTractModel8 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel8 = null;
            }
            int hopSize = sharedTractModel8.getAtlas().getHopSize();
            RealtimeDspPipeline realtimeDspPipeline = this.pipeline;
            if (realtimeDspPipeline == null) {
                Intrinsics.throwUninitializedPropertyAccessException("pipeline");
                realtimeDspPipeline = null;
            }
            this.capture = new AudioCaptureEngine(applicationContext2, sampleRateHz, frameSize, hopSize, new AnatomyBenchActivity$onCreate$7(realtimeDspPipeline), new Function1() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda21
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function1
                public final Object invoke(Object obj) {
                    Unit onCreate$lambda$11;
                    onCreate$lambda$11 = AnatomyBenchActivity.onCreate$lambda$11(AnatomyBenchActivity.this, (String) obj);
                    return onCreate$lambda$11;
                }
            });
            SharedTractModel sharedTractModel9 = this.model;
            if (sharedTractModel9 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel9 = null;
            }
            int sampleRateHz2 = sharedTractModel9.getAtlas().getSampleRateHz();
            SharedTractModel sharedTractModel10 = this.model;
            if (sharedTractModel10 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel10 = null;
            }
            int hopSize2 = sharedTractModel10.getAtlas().getHopSize();
            RealtimeDspPipeline realtimeDspPipeline2 = this.pipeline;
            if (realtimeDspPipeline2 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("pipeline");
                realtimeDspPipeline2 = null;
            }
            this.replay = new ReplayController(sampleRateHz2, hopSize2, new AnatomyBenchActivity$onCreate$9(realtimeDspPipeline2));
            this.calibration = new SingerCalibrationController(new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda22
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function0
                public final Object invoke() {
                    Unit onCreate$lambda$12;
                    onCreate$lambda$12 = AnatomyBenchActivity.onCreate$lambda$12(AnatomyBenchActivity.this);
                    return onCreate$lambda$12;
                }
            }, new AnatomyBenchActivity$onCreate$11(DiagnosticsRuntime.INSTANCE));
            buildUi();
            DiagnosticsRuntime.INSTANCE.updateRendererMode("shared_mri_canvas");
            DiagnosticsRuntime diagnosticsRuntime2 = DiagnosticsRuntime.INSTANCE;
            Pair[] pairArr = new Pair[3];
            SharedTractModel sharedTractModel11 = this.model;
            if (sharedTractModel11 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
                sharedTractModel11 = null;
            }
            pairArr[0] = TuplesKt.to("model_id", sharedTractModel11.getId());
            SharedTractModel sharedTractModel12 = this.model;
            if (sharedTractModel12 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("model");
            } else {
                sharedTractModel3 = sharedTractModel12;
            }
            pairArr[1] = TuplesKt.to("source_model_sha256", sharedTractModel3.getAtlas().getAtlasSourceModelSha256());
            pairArr[2] = TuplesKt.to("scientific_release_ready", "false");
            DiagnosticsRuntime.log$default(diagnosticsRuntime2, "model", "shared_model_loaded", "One shared model for manual/live/render/audio", MapsKt.mapOf(pairArr), null, 16, null);
        } catch (Exception e) {
            TextView textView = new TextView(this);
            textView.setText("MRI model could not be verified. Audio is disabled.\n" + e.getMessage());
            textView.setPadding(24, 64, 24, 24);
            setContentView(textView);
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onCreate$lambda$3(final AnatomyBenchActivity anatomyBenchActivity, int i) {
        if (i < 0) {
            anatomyBenchActivity.runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda1
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // java.lang.Runnable
                public final void run() {
                    AnatomyBenchActivity.onCreate$lambda$3$lambda$2(AnatomyBenchActivity.this);
                }
            });
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onCreate$lambda$3$lambda$2(AnatomyBenchActivity anatomyBenchActivity) {
        anatomyBenchActivity.stopTone();
        anatomyBenchActivity.statusMessage = "Audio focus lost; playback stopped.";
        anatomyBenchActivity.sendStatus();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit onCreate$lambda$5(final AnatomyBenchActivity anatomyBenchActivity, final String message) {
        Intrinsics.checkNotNullParameter(message, "message");
        anatomyBenchActivity.runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda10
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                AnatomyBenchActivity.onCreate$lambda$5$lambda$4(AnatomyBenchActivity.this, message);
            }
        });
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onCreate$lambda$5$lambda$4(AnatomyBenchActivity anatomyBenchActivity, String str) {
        anatomyBenchActivity.playWanted = false;
        anatomyBenchActivity.statusMessage = str;
        anatomyBenchActivity.sendStatus();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit onCreate$lambda$7(final AnatomyBenchActivity anatomyBenchActivity, final VocalAcousticsState observation, final long j) {
        Intrinsics.checkNotNullParameter(observation, "observation");
        anatomyBenchActivity.runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda2
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                AnatomyBenchActivity.onCreate$lambda$7$lambda$6(j, anatomyBenchActivity, observation);
            }
        });
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onCreate$lambda$7$lambda$6(long j, AnatomyBenchActivity anatomyBenchActivity, VocalAcousticsState vocalAcousticsState) {
        if (j == anatomyBenchActivity.epoch.get()) {
            anatomyBenchActivity.acceptObservation(vocalAcousticsState);
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit onCreate$lambda$9(final AnatomyBenchActivity anatomyBenchActivity, final String message) {
        Intrinsics.checkNotNullParameter(message, "message");
        anatomyBenchActivity.runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda3
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                AnatomyBenchActivity.onCreate$lambda$9$lambda$8(AnatomyBenchActivity.this, message);
            }
        });
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onCreate$lambda$9$lambda$8(AnatomyBenchActivity anatomyBenchActivity, String str) {
        anatomyBenchActivity.statusMessage = str;
        anatomyBenchActivity.sendStatus();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit onCreate$lambda$11(final AnatomyBenchActivity anatomyBenchActivity, final String message) {
        Intrinsics.checkNotNullParameter(message, "message");
        anatomyBenchActivity.runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda12
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                AnatomyBenchActivity.onCreate$lambda$11$lambda$10(AnatomyBenchActivity.this, message);
            }
        });
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void onCreate$lambda$11$lambda$10(AnatomyBenchActivity anatomyBenchActivity, String str) {
        anatomyBenchActivity.stopInput();
        anatomyBenchActivity.statusMessage = str;
        anatomyBenchActivity.sendStatus();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit onCreate$lambda$12(AnatomyBenchActivity anatomyBenchActivity) {
        RealtimeDspPipeline realtimeDspPipeline = anatomyBenchActivity.pipeline;
        if (realtimeDspPipeline == null) {
            Intrinsics.throwUninitializedPropertyAccessException("pipeline");
            realtimeDspPipeline = null;
        }
        realtimeDspPipeline.requestNoiseLearning(140);
        return Unit.INSTANCE;
    }

    private final void buildUi() {
        AnatomyBenchActivity anatomyBenchActivity = this;
        WebView webView = new WebView(anatomyBenchActivity);
        this.web = webView;
        webView.setBackgroundColor(Color.rgb(9, 21, 29));
        WebView webView2 = this.web;
        WebView webView3 = null;
        if (webView2 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("web");
            webView2 = null;
        }
        WebSettings settings = webView2.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setAllowFileAccess(false);
        settings.setAllowContentAccess(false);
        settings.setBlockNetworkLoads(true);
        settings.setDomStorageEnabled(false);
        settings.setMixedContentMode(1);
        settings.setSupportMultipleWindows(false);
        WebView webView4 = this.web;
        if (webView4 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("web");
            webView4 = null;
        }
        webView4.setWebChromeClient(new WebChromeClient() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$buildUi$2
            @Override // android.webkit.WebChromeClient
            public void onPermissionRequest(PermissionRequest request) {
                Intrinsics.checkNotNullParameter(request, "request");
                request.deny();
            }
        });
        WebView webView5 = this.web;
        if (webView5 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("web");
            webView5 = null;
        }
        webView5.setWebViewClient(new AnatomyBenchActivity$buildUi$3(this));
        WebView webView6 = this.web;
        if (webView6 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("web");
            webView6 = null;
        }
        webView6.addJavascriptInterface(new Commands(), "NativeLab");
        LinearLayout linearLayout = new LinearLayout(anatomyBenchActivity);
        linearLayout.setOrientation(1);
        linearLayout.setBackgroundColor(Color.rgb(9, 21, 29));
        if (Build.VERSION.SDK_INT >= 30) {
            linearLayout.setOnApplyWindowInsetsListener(new View.OnApplyWindowInsetsListener() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda4
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // android.view.View.OnApplyWindowInsetsListener
                public final WindowInsets onApplyWindowInsets(View view, WindowInsets windowInsets) {
                    WindowInsets buildUi$lambda$15;
                    buildUi$lambda$15 = AnatomyBenchActivity.buildUi$lambda$15(view, windowInsets);
                    return buildUi$lambda$15;
                }
            });
            WindowInsetsController insetsController = getWindow().getInsetsController();
            if (insetsController != null) {
                insetsController.setSystemBarsAppearance(0, 24);
            }
        }
        LinearLayout linearLayout2 = new LinearLayout(anatomyBenchActivity);
        linearLayout2.setGravity(16);
        int i = (int) (12 * linearLayout2.getResources().getDisplayMetrics().density);
        linearLayout2.setPadding(i, 0, i, 0);
        TextView textView = new TextView(anatomyBenchActivity);
        textView.setText("Vocal Tract Lab · 0.10.0");
        textView.setTextSize(14.0f);
        textView.setTextColor(Color.rgb(238, 244, 237));
        linearLayout2.addView(textView, new LinearLayout.LayoutParams(0, -2, 1.0f));
        Button button = new Button(anatomyBenchActivity);
        button.setText("Diagnostics");
        button.setOnClickListener(new View.OnClickListener() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda5
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                AnatomyBenchActivity.buildUi$lambda$19$lambda$18$lambda$17(AnatomyBenchActivity.this, view);
            }
        });
        linearLayout2.addView(button);
        linearLayout.addView(linearLayout2);
        WebView webView7 = this.web;
        if (webView7 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("web");
            webView7 = null;
        }
        linearLayout.addView(webView7, new LinearLayout.LayoutParams(-1, 0, 1.0f));
        setContentView(linearLayout);
        linearLayout.requestApplyInsets();
        WebView webView8 = this.web;
        if (webView8 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("web");
        } else {
            webView3 = webView8;
        }
        webView3.loadUrl(LabAssets.PAGE);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final WindowInsets buildUi$lambda$15(View view, WindowInsets insets) {
        Intrinsics.checkNotNullParameter(view, "view");
        Intrinsics.checkNotNullParameter(insets, "insets");
        Insets insets2 = insets.getInsets(WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout() | WindowInsets.Type.ime());
        Intrinsics.checkNotNullExpressionValue(insets2, "getInsets(...)");
        view.setPadding(insets2.left, insets2.top, insets2.right, insets2.bottom);
        return WindowInsets.CONSUMED;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void buildUi$lambda$19$lambda$18$lambda$17(AnatomyBenchActivity anatomyBenchActivity, View view) {
        anatomyBenchActivity.stopEverything();
        anatomyBenchActivity.startActivity(new Intent(anatomyBenchActivity, (Class<?>) AdminConsoleActivity.class));
    }

    /* JADX DEBUG: Don't trust debug lines info. Repeating lines: [131=7] */
    /* JADX INFO: Access modifiers changed from: private */
    /* JADX WARN: Can't fix incorrect switch cases order, some code will duplicate */
    /* JADX WARN: Removed duplicated region for block: B:94:0x01f5  */
    /* JADX WARN: Removed duplicated region for block: B:9:0x0225  */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public final void handle(final LabCommand c) {
        SingerCalibrationController singerCalibrationController;
        WebView webView;
        AudioFocusRequest audioFocusRequest;
        RealtimeDspPipeline realtimeDspPipeline;
        String type = c.getType();
        switch (type.hashCode()) {
            case -1129246009:
                if (type.equals("calibrate")) {
                    if (Intrinsics.areEqual(this.inputMode, "mic")) {
                        SingerCalibrationController singerCalibrationController2 = this.calibration;
                        if (singerCalibrationController2 == null) {
                            Intrinsics.throwUninitializedPropertyAccessException("calibration");
                            singerCalibrationController2 = null;
                        }
                        if (singerCalibrationController2.isActive()) {
                            SingerCalibrationController singerCalibrationController3 = this.calibration;
                            if (singerCalibrationController3 == null) {
                                Intrinsics.throwUninitializedPropertyAccessException("calibration");
                                singerCalibrationController = null;
                            } else {
                                singerCalibrationController = singerCalibrationController3;
                            }
                            this.calibrationText = singerCalibrationController.cancel().getPrompt();
                        } else {
                            SingerCalibrationController singerCalibrationController4 = this.calibration;
                            if (singerCalibrationController4 == null) {
                                Intrinsics.throwUninitializedPropertyAccessException("calibration");
                                singerCalibrationController4 = null;
                            }
                            this.calibrationText = SingerCalibrationController.start$default(singerCalibrationController4, 0L, 1, null).getPrompt();
                        }
                    } else {
                        this.statusMessage = "Start listening before calibration.";
                    }
                    sendStatus();
                    if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                        acknowledge(c.getRequestId());
                        break;
                    }
                }
                if (!Intrinsics.areEqual(this.inputMode, "idle")) {
                    stopInput();
                    this.statusMessage = "Manual tuning · microphone inactive.";
                }
                submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda9
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit handle$lambda$25;
                        handle$lambda$25 = AnatomyBenchActivity.handle$lambda$25(LabCommand.this, this);
                        return handle$lambda$25;
                    }
                }, 1, null);
                if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                }
                break;
            case -1102508601:
                if (type.equals("listen")) {
                    stopEverything();
                    if (checkSelfPermission("android.permission.RECORD_AUDIO") == 0) {
                        beginCapture();
                    } else {
                        this.permissionEpoch = Long.valueOf(this.epoch.get());
                        requestPermissions(new String[]{"android.permission.RECORD_AUDIO"}, 70);
                    }
                    if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                    }
                }
                if (!Intrinsics.areEqual(this.inputMode, "idle")) {
                }
                submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda9
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit handle$lambda$25;
                        handle$lambda$25 = AnatomyBenchActivity.handle$lambda$25(LabCommand.this, this);
                        return handle$lambda$25;
                    }
                }, 1, null);
                if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                }
                break;
            case 3079651:
                if (type.equals("demo")) {
                    stopEverything();
                    this.inputMode = "demo";
                    this.statusMessage = "10-second replay generated by this same tract model; no microphone.";
                    final long j = this.epoch.get();
                    RealtimeDspPipeline realtimeDspPipeline2 = this.pipeline;
                    if (realtimeDspPipeline2 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("pipeline");
                        realtimeDspPipeline2 = null;
                    }
                    if (!realtimeDspPipeline2.start(j, this.currentCoordinates)) {
                        this.inputMode = "idle";
                        this.statusMessage = "Analysis is still stopping; tap Replay again.";
                        sendStatus();
                        acknowledge(c.getRequestId());
                        break;
                    } else {
                        ReplayController replayController = this.replay;
                        if (replayController == null) {
                            Intrinsics.throwUninitializedPropertyAccessException("replay");
                            replayController = null;
                        }
                        SharedReplay sharedReplay = SharedReplay.INSTANCE;
                        AdditiveSynthState additiveSynthState = this.currentSound;
                        SharedTractModel sharedTractModel = this.model;
                        if (sharedTractModel == null) {
                            Intrinsics.throwUninitializedPropertyAccessException("model");
                            sharedTractModel = null;
                        }
                        int sampleRateHz = sharedTractModel.getAtlas().getSampleRateHz();
                        SharedTractModel sharedTractModel2 = this.model;
                        if (sharedTractModel2 == null) {
                            Intrinsics.throwUninitializedPropertyAccessException("model");
                            sharedTractModel2 = null;
                        }
                        int frameSize = sharedTractModel2.getAtlas().getFrameSize();
                        SharedTractModel sharedTractModel3 = this.model;
                        if (sharedTractModel3 == null) {
                            Intrinsics.throwUninitializedPropertyAccessException("model");
                            sharedTractModel3 = null;
                        }
                        replayController.play(sharedReplay.frames(additiveSynthState, sampleRateHz, frameSize, sharedTractModel3.getAtlas().getHopSize()), true);
                        WebView webView2 = this.web;
                        if (webView2 == null) {
                            Intrinsics.throwUninitializedPropertyAccessException("web");
                            webView = null;
                        } else {
                            webView = webView2;
                        }
                        webView.postDelayed(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda8
                            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                            @Override // java.lang.Runnable
                            public final void run() {
                                AnatomyBenchActivity.handle$lambda$22(AnatomyBenchActivity.this, j);
                            }
                        }, 10500L);
                        sendStatus();
                        if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                        }
                    }
                }
                if (!Intrinsics.areEqual(this.inputMode, "idle")) {
                }
                submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda9
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit handle$lambda$25;
                        handle$lambda$25 = AnatomyBenchActivity.handle$lambda$25(LabCommand.this, this);
                        return handle$lambda$25;
                    }
                }, 1, null);
                if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                }
                break;
            case 3443508:
                if (type.equals("play")) {
                    stopInput();
                    AudioManager audioManager = this.audioManager;
                    if (audioManager == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("audioManager");
                        audioManager = null;
                    }
                    AudioFocusRequest audioFocusRequest2 = this.focusRequest;
                    if (audioFocusRequest2 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("focusRequest");
                        audioFocusRequest = null;
                    } else {
                        audioFocusRequest = audioFocusRequest2;
                    }
                    if (audioManager.requestAudioFocus(audioFocusRequest) != 1) {
                        this.statusMessage = "Audio focus unavailable.";
                        sendStatus();
                        acknowledge(c.getRequestId());
                        break;
                    } else {
                        this.playWanted = true;
                        this.statusMessage = "Start at low phone volume.";
                        submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda7
                            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                            @Override // kotlin.jvm.functions.Function0
                            public final Object invoke() {
                                Unit unit;
                                unit = Unit.INSTANCE;
                                return unit;
                            }
                        }, 1, null);
                        if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                        }
                    }
                }
                if (!Intrinsics.areEqual(this.inputMode, "idle")) {
                }
                submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda9
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit handle$lambda$25;
                        handle$lambda$25 = AnatomyBenchActivity.handle$lambda$25(LabCommand.this, this);
                        return handle$lambda$25;
                    }
                }, 1, null);
                if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                }
                break;
            case 3540994:
                if (type.equals("stop")) {
                    stopEverything();
                    this.statusMessage = "Stopped.";
                    sendStatus();
                    if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                    }
                }
                if (!Intrinsics.areEqual(this.inputMode, "idle")) {
                }
                submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda9
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit handle$lambda$25;
                        handle$lambda$25 = AnatomyBenchActivity.handle$lambda$25(LabCommand.this, this);
                        return handle$lambda$25;
                    }
                }, 1, null);
                if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                }
                break;
            case 104998682:
                if (type.equals("noise")) {
                    if (Intrinsics.areEqual(this.inputMode, "mic")) {
                        RealtimeDspPipeline realtimeDspPipeline3 = this.pipeline;
                        if (realtimeDspPipeline3 == null) {
                            Intrinsics.throwUninitializedPropertyAccessException("pipeline");
                            realtimeDspPipeline = null;
                        } else {
                            realtimeDspPipeline = realtimeDspPipeline3;
                        }
                        realtimeDspPipeline.requestNoiseLearning(100);
                        this.statusMessage = "Learning normal background. Stay quiet for about 3 seconds.";
                    }
                    sendStatus();
                    if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                    }
                }
                if (!Intrinsics.areEqual(this.inputMode, "idle")) {
                }
                submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda9
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit handle$lambda$25;
                        handle$lambda$25 = AnatomyBenchActivity.handle$lambda$25(LabCommand.this, this);
                        return handle$lambda$25;
                    }
                }, 1, null);
                if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                }
                break;
            case 108386723:
                if (type.equals("ready")) {
                    submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda6
                        /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                        @Override // kotlin.jvm.functions.Function0
                        public final Object invoke() {
                            Unit unit;
                            unit = Unit.INSTANCE;
                            return unit;
                        }
                    }, 1, null);
                    if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                    }
                }
                if (!Intrinsics.areEqual(this.inputMode, "idle")) {
                }
                submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda9
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit handle$lambda$25;
                        handle$lambda$25 = AnatomyBenchActivity.handle$lambda$25(LabCommand.this, this);
                        return handle$lambda$25;
                    }
                }, 1, null);
                if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                }
                break;
            default:
                if (!Intrinsics.areEqual(this.inputMode, "idle")) {
                }
                submit$default(this, null, Integer.valueOf(c.getRequestId()), new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda9
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit handle$lambda$25;
                        handle$lambda$25 = AnatomyBenchActivity.handle$lambda$25(LabCommand.this, this);
                        return handle$lambda$25;
                    }
                }, 1, null);
                if (CollectionsKt.listOf((Object[]) new String[]{"stop", "listen", "demo", "noise", "calibrate"}).contains(c.getType())) {
                }
                break;
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void handle$lambda$22(AnatomyBenchActivity anatomyBenchActivity, long j) {
        if (!anatomyBenchActivity.destroyed && Intrinsics.areEqual(anatomyBenchActivity.inputMode, "demo") && anatomyBenchActivity.epoch.get() == j) {
            anatomyBenchActivity.stopInput();
            anatomyBenchActivity.statusMessage = "Model replay complete.";
            anatomyBenchActivity.sendStatus();
        }
    }

    /* JADX DEBUG: Don't trust debug lines info. Repeating lines: [162=8] */
    /* JADX INFO: Access modifiers changed from: private */
    /* JADX WARN: Code restructure failed: missing block: B:51:0x00d1, code lost:
    
        if (r5 == null) goto L57;
     */
    /* JADX WARN: Failed to restore switch over string. Please report as a decompilation issue */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public static final Unit handle$lambda$25(LabCommand labCommand, AnatomyBenchActivity anatomyBenchActivity) {
        float[] fArr;
        float[] floatArray;
        String type = labCommand.getType();
        SharedLabState sharedLabState = null;
        SharedLabState sharedLabState2 = null;
        SharedLabState sharedLabState3 = null;
        SharedLabState sharedLabState4 = null;
        SharedLabState sharedLabState5 = null;
        SharedLabState sharedLabState6 = null;
        SharedLabState sharedLabState7 = null;
        Object obj = null;
        switch (type.hashCode()) {
            case -1081267614:
                if (type.equals("master")) {
                    SharedLabState sharedLabState8 = anatomyBenchActivity.state;
                    if (sharedLabState8 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                    } else {
                        sharedLabState = sharedLabState8;
                    }
                    Double value = labCommand.getValue();
                    if (value == null) {
                        throw new IllegalArgumentException("Required value was null.".toString());
                    }
                    sharedLabState.setMaster((float) value.doubleValue());
                    break;
                }
                break;
            case -980098337:
                if (type.equals("preset")) {
                    SharedLabState sharedLabState9 = anatomyBenchActivity.state;
                    if (sharedLabState9 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                        sharedLabState9 = null;
                    }
                    Iterator<T> it = SharedLabJson.INSTANCE.getPresets().iterator();
                    while (true) {
                        if (it.hasNext()) {
                            Object next = it.next();
                            if (Intrinsics.areEqual(((Triple) next).getFirst(), labCommand.getId())) {
                                obj = next;
                            }
                        }
                    }
                    Triple triple = (Triple) obj;
                    if (triple != null && (fArr = (float[]) triple.getThird()) != null) {
                        sharedLabState9.tune(fArr);
                        break;
                    } else {
                        throw new IllegalStateException("Unknown vowel".toString());
                    }
                }
                break;
            case -934396757:
                if (type.equals("retune")) {
                    SharedLabState sharedLabState10 = anatomyBenchActivity.state;
                    if (sharedLabState10 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                    } else {
                        sharedLabState7 = sharedLabState10;
                    }
                    sharedLabState7.resetSource();
                    break;
                }
                break;
            case -792934015:
                if (type.equals("partial")) {
                    SharedLabState sharedLabState11 = anatomyBenchActivity.state;
                    if (sharedLabState11 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                    } else {
                        sharedLabState6 = sharedLabState11;
                    }
                    Integer index = labCommand.getIndex();
                    if (index == null) {
                        throw new IllegalArgumentException("Required value was null.".toString());
                    }
                    int intValue = index.intValue();
                    Double value2 = labCommand.getValue();
                    if (value2 == null) {
                        throw new IllegalArgumentException("Required value was null.".toString());
                    }
                    sharedLabState6.setPartial(intValue, (float) value2.doubleValue());
                    break;
                }
                break;
            case -677443933:
                if (type.equals("formant")) {
                    Integer index2 = labCommand.getIndex();
                    if (index2 == null) {
                        throw new IllegalArgumentException("Required value was null.".toString());
                    }
                    int intValue2 = index2.intValue();
                    if (intValue2 < 0 || intValue2 >= 3) {
                        throw new IllegalArgumentException("Failed requirement.".toString());
                    }
                    SharedLabState sharedLabState12 = anatomyBenchActivity.state;
                    if (sharedLabState12 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                        sharedLabState12 = null;
                    }
                    float[] targets = sharedLabState12.getTargets();
                    if (targets != null) {
                        floatArray = Arrays.copyOf(targets, targets.length);
                        Intrinsics.checkNotNullExpressionValue(floatArray, "copyOf(...)");
                        break;
                    }
                    SharedTractModel sharedTractModel = anatomyBenchActivity.model;
                    if (sharedTractModel == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("model");
                        sharedTractModel = null;
                    }
                    SharedLabState sharedLabState13 = anatomyBenchActivity.state;
                    if (sharedLabState13 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                        sharedLabState13 = null;
                    }
                    List<Double> take = ArraysKt.take(sharedTractModel.response(sharedLabState13.getCoefficients()).getPeaksHz(), 3);
                    ArrayList arrayList = new ArrayList(CollectionsKt.collectionSizeOrDefault(take, 10));
                    Iterator<T> it2 = take.iterator();
                    while (it2.hasNext()) {
                        arrayList.add(Float.valueOf((float) ((Number) it2.next()).doubleValue()));
                    }
                    floatArray = CollectionsKt.toFloatArray(arrayList);
                    Double value3 = labCommand.getValue();
                    if (value3 == null) {
                        throw new IllegalArgumentException("Required value was null.".toString());
                    }
                    floatArray[intValue2] = (float) value3.doubleValue();
                    SharedLabState sharedLabState14 = anatomyBenchActivity.state;
                    if (sharedLabState14 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                    } else {
                        sharedLabState5 = sharedLabState14;
                    }
                    sharedLabState5.tune(floatArray);
                    break;
                }
                break;
            case 106677056:
                if (type.equals("pitch")) {
                    SharedLabState sharedLabState15 = anatomyBenchActivity.state;
                    if (sharedLabState15 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                    } else {
                        sharedLabState4 = sharedLabState15;
                    }
                    Double value4 = labCommand.getValue();
                    if (value4 == null) {
                        throw new IllegalArgumentException("Required value was null.".toString());
                    }
                    sharedLabState4.setPitch((float) value4.doubleValue());
                    break;
                }
                break;
            case 108404047:
                if (type.equals("reset")) {
                    SharedLabState sharedLabState16 = anatomyBenchActivity.state;
                    if (sharedLabState16 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                    } else {
                        sharedLabState3 = sharedLabState16;
                    }
                    sharedLabState3.reset();
                    break;
                }
                break;
            case 198931832:
                if (type.equals("coordinate")) {
                    SharedLabState sharedLabState17 = anatomyBenchActivity.state;
                    if (sharedLabState17 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("state");
                    } else {
                        sharedLabState2 = sharedLabState17;
                    }
                    Integer index3 = labCommand.getIndex();
                    if (index3 == null) {
                        throw new IllegalArgumentException("Required value was null.".toString());
                    }
                    int intValue3 = index3.intValue();
                    Double value5 = labCommand.getValue();
                    if (value5 == null) {
                        throw new IllegalArgumentException("Required value was null.".toString());
                    }
                    sharedLabState2.setMode(intValue3, (float) value5.doubleValue());
                    break;
                }
                break;
        }
        return Unit.INSTANCE;
    }

    private final void acknowledge(final int id) {
        runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda16
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.lang.Runnable
            public final void run() {
                AnatomyBenchActivity.acknowledge$lambda$26(AnatomyBenchActivity.this, id);
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void acknowledge$lambda$26(AnatomyBenchActivity anatomyBenchActivity, int i) {
        if (anatomyBenchActivity.trusted && anatomyBenchActivity.resumed && !anatomyBenchActivity.destroyed) {
            WebView webView = anatomyBenchActivity.web;
            if (webView == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
                webView = null;
            }
            webView.evaluateJavascript("window.LabHost && window.LabHost.ack(" + i + ");", null);
        }
    }

    /* JADX DEBUG: Multi-variable search result rejected for r0v0, resolved type: org.vocaltract.pixel.AnatomyBenchActivity */
    /* JADX WARN: Multi-variable type inference failed */
    static /* synthetic */ void submit$default(AnatomyBenchActivity anatomyBenchActivity, Function0 function0, Integer num, Function0 function02, int i, Object obj) {
        if ((i & 1) != 0) {
            function0 = new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda15
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function0
                public final Object invoke() {
                    Unit unit;
                    unit = Unit.INSTANCE;
                    return unit;
                }
            };
        }
        if ((i & 2) != 0) {
            num = null;
        }
        anatomyBenchActivity.submit(function0, num, function02);
    }

    private final void submit(final Function0<Unit> onDone, final Integer requestId, final Function0<Unit> change) {
        final long j = this.epoch.get();
        try {
            this.executor.execute(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda11
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // java.lang.Runnable
                public final void run() {
                    AnatomyBenchActivity.submit$lambda$31(AnatomyBenchActivity.this, j, change, onDone, requestId);
                }
            });
        } catch (RejectedExecutionException unused) {
            onDone.invoke();
            if (requestId != null) {
                acknowledge(requestId.intValue());
            }
            this.statusMessage = "Model busy; try the control again.";
            sendStatus();
        }
    }

    /* JADX DEBUG: Another duplicated slice has different insns count: {[INVOKE]}, finally: {[INVOKE, CHECK_CAST, INVOKE, INVOKE, IF] complete} */
    /* JADX INFO: Access modifiers changed from: private */
    public static final void submit$lambda$31(final AnatomyBenchActivity anatomyBenchActivity, final long j, Function0 function0, Function0 function02, Integer num) {
        try {
            try {
            } catch (Exception e) {
                anatomyBenchActivity.runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda24
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // java.lang.Runnable
                    public final void run() {
                        AnatomyBenchActivity.submit$lambda$31$lambda$29(j, anatomyBenchActivity, e);
                    }
                });
                function02.invoke();
                if (num == null) {
                    return;
                }
            }
            if (!anatomyBenchActivity.destroyed && j == anatomyBenchActivity.epoch.get()) {
                function0.invoke();
                SharedLabJson sharedLabJson = SharedLabJson.INSTANCE;
                SharedLabState sharedLabState = anatomyBenchActivity.state;
                SharedLabState sharedLabState2 = null;
                if (sharedLabState == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("state");
                    sharedLabState = null;
                }
                final String jSONObject = sharedLabJson.snapshot(sharedLabState).toString();
                Intrinsics.checkNotNullExpressionValue(jSONObject, "toString(...)");
                SharedLabState sharedLabState3 = anatomyBenchActivity.state;
                if (sharedLabState3 == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("state");
                    sharedLabState3 = null;
                }
                final AdditiveSynthState sound = sharedLabState3.sound();
                SharedLabState sharedLabState4 = anatomyBenchActivity.state;
                if (sharedLabState4 == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("state");
                    sharedLabState4 = null;
                }
                float[] coefficients = sharedLabState4.getCoefficients();
                final float[] copyOf = Arrays.copyOf(coefficients, coefficients.length);
                Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
                SharedLabJson sharedLabJson2 = SharedLabJson.INSTANCE;
                SharedLabState sharedLabState5 = anatomyBenchActivity.state;
                if (sharedLabState5 == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("state");
                } else {
                    sharedLabState2 = sharedLabState5;
                }
                anatomyBenchActivity.getSharedPreferences("shared_mri_lab", 0).edit().putString("state", sharedLabJson2.save(sharedLabState2)).apply();
                DiagnosticsRuntime.INSTANCE.setSharedModelSnapshot(jSONObject);
                anatomyBenchActivity.runOnUiThread(new Runnable() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda23
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // java.lang.Runnable
                    public final void run() {
                        AnatomyBenchActivity.submit$lambda$31$lambda$28(AnatomyBenchActivity.this, j, sound, copyOf, jSONObject);
                    }
                });
                function02.invoke();
                if (num == null) {
                    return;
                }
                anatomyBenchActivity.acknowledge(num.intValue());
                return;
            }
            function02.invoke();
            if (num != null) {
                anatomyBenchActivity.acknowledge(num.intValue());
            }
        } catch (Throwable th) {
            function02.invoke();
            if (num != null) {
                anatomyBenchActivity.acknowledge(num.intValue());
            }
            throw th;
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void submit$lambda$31$lambda$28(AnatomyBenchActivity anatomyBenchActivity, long j, AdditiveSynthState additiveSynthState, float[] fArr, String str) {
        if (!anatomyBenchActivity.destroyed && anatomyBenchActivity.resumed && anatomyBenchActivity.trusted && j == anatomyBenchActivity.epoch.get()) {
            anatomyBenchActivity.currentSound = additiveSynthState;
            anatomyBenchActivity.currentCoordinates = fArr;
            anatomyBenchActivity.latestSnapshot = str;
            if (anatomyBenchActivity.playWanted && Intrinsics.areEqual(anatomyBenchActivity.inputMode, "idle")) {
                AdditiveSynthEngine additiveSynthEngine = anatomyBenchActivity.synth;
                if (additiveSynthEngine == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("synth");
                    additiveSynthEngine = null;
                }
                AdditiveSynthController controller = additiveSynthEngine.getController();
                AdditiveSynthEngine additiveSynthEngine2 = anatomyBenchActivity.synth;
                if (additiveSynthEngine2 == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("synth");
                    additiveSynthEngine2 = null;
                }
                controller.replace(additiveSynthState.withPlaying(additiveSynthEngine2.isRunning()));
                AdditiveSynthEngine additiveSynthEngine3 = anatomyBenchActivity.synth;
                if (additiveSynthEngine3 == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("synth");
                    additiveSynthEngine3 = null;
                }
                if (!additiveSynthEngine3.start()) {
                    anatomyBenchActivity.playWanted = false;
                }
            }
            DiagnosticsRuntime diagnosticsRuntime = DiagnosticsRuntime.INSTANCE;
            AdditiveSynthEngine additiveSynthEngine4 = anatomyBenchActivity.synth;
            if (additiveSynthEngine4 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("synth");
                additiveSynthEngine4 = null;
            }
            diagnosticsRuntime.updateSynthesizerActive(additiveSynthEngine4.isRunning());
            WebView webView = anatomyBenchActivity.web;
            if (webView == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
                webView = null;
            }
            webView.evaluateJavascript("window.LabHost && window.LabHost.frame(" + str + ");", null);
            anatomyBenchActivity.sendStatus();
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void submit$lambda$31$lambda$29(long j, AnatomyBenchActivity anatomyBenchActivity, Exception exc) {
        if (j == anatomyBenchActivity.epoch.get()) {
            String message = exc.getMessage();
            if (message == null) {
                message = "Model update failed";
            }
            anatomyBenchActivity.statusMessage = message;
            anatomyBenchActivity.sendStatus();
        }
    }

    private final void beginCapture() {
        if (!this.resumed || this.destroyed) {
            return;
        }
        RealtimeDspPipeline realtimeDspPipeline = this.pipeline;
        RealtimeDspPipeline realtimeDspPipeline2 = null;
        if (realtimeDspPipeline == null) {
            Intrinsics.throwUninitializedPropertyAccessException("pipeline");
            realtimeDspPipeline = null;
        }
        if (!realtimeDspPipeline.start(this.epoch.get(), this.currentCoordinates)) {
            this.statusMessage = "Analysis is still stopping; tap Listen again.";
            sendStatus();
            return;
        }
        AudioCaptureEngine audioCaptureEngine = this.capture;
        if (audioCaptureEngine == null) {
            Intrinsics.throwUninitializedPropertyAccessException("capture");
            audioCaptureEngine = null;
        }
        if (audioCaptureEngine.start()) {
            this.inputMode = "mic";
            this.statusMessage = "Listening · same MRI model · no raw recording stored.";
            DiagnosticsRuntime.INSTANCE.updateMicrophoneGranted(true);
        } else {
            RealtimeDspPipeline realtimeDspPipeline3 = this.pipeline;
            if (realtimeDspPipeline3 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("pipeline");
            } else {
                realtimeDspPipeline2 = realtimeDspPipeline3;
            }
            realtimeDspPipeline2.stop(false);
        }
        sendStatus();
    }

    @Override // android.app.Activity
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        Integer firstOrNull;
        Intrinsics.checkNotNullParameter(permissions, "permissions");
        Intrinsics.checkNotNullParameter(grantResults, "grantResults");
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == 70) {
            Long l = this.permissionEpoch;
            this.permissionEpoch = null;
            if (l != null) {
                if (l.longValue() == this.epoch.get() && this.resumed && (firstOrNull = ArraysKt.firstOrNull(grantResults)) != null && firstOrNull.intValue() == 0) {
                    beginCapture();
                    return;
                }
            }
            this.statusMessage = "Microphone inactive. Tap Listen to start when permitted.";
            sendStatus();
        }
    }

    private final void acceptObservation(final VocalAcousticsState observation) {
        ArrayList emptyList;
        SingerCalibrationController singerCalibrationController;
        if (!this.resumed || Intrinsics.areEqual(this.inputMode, "idle") || this.destroyed) {
            return;
        }
        DiagnosticsRuntime.INSTANCE.updateState(observation);
        SingerCalibrationController singerCalibrationController2 = this.calibration;
        if (singerCalibrationController2 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("calibration");
            singerCalibrationController2 = null;
        }
        if (singerCalibrationController2.isActive()) {
            SingerCalibrationController singerCalibrationController3 = this.calibration;
            if (singerCalibrationController3 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("calibration");
                singerCalibrationController = null;
            } else {
                singerCalibrationController = singerCalibrationController3;
            }
            CalibrationProgress accept$default = SingerCalibrationController.accept$default(singerCalibrationController, observation, 0L, 2, null);
            this.calibrationText = (accept$default.getStepIndex() + 1) + "/" + accept$default.getStepCount() + ": " + accept$default.getPrompt() + " (" + ((int) (accept$default.getFraction() * 100)) + "%)";
        }
        long elapsedRealtime = SystemClock.elapsedRealtime();
        if (elapsedRealtime - this.lastUiFrame < 96 || !this.livePending.compareAndSet(false, true)) {
            return;
        }
        this.lastUiFrame = elapsedRealtime;
        final AcousticEstimate acoustic = observation.getAcoustic();
        JSONObject put = new JSONObject().put("f0", acoustic.getF0Hz() != null ? Double.valueOf(r3.floatValue()) : JSONObject.NULL).put("snr", acoustic.getSnrDb()).put("noise", acoustic.getNoiseState()).put("pitchDecision", acoustic.getPitchDecision()).put("confidence", observation.getTract().getConfidence()).put("abstained", observation.getTract().getAbstained()).put("reason", observation.getTract().getAbstentionReason());
        if (!acoustic.getVoiced()) {
            emptyList = CollectionsKt.emptyList();
        } else {
            List take = CollectionsKt.take(acoustic.getFormantsHz(), RangesKt.coerceAtMost(acoustic.getFormantCandidatesHz().size(), 3));
            ArrayList arrayList = new ArrayList(CollectionsKt.collectionSizeOrDefault(take, 10));
            Iterator it = take.iterator();
            while (it.hasNext()) {
                arrayList.add(Double.valueOf(((Number) it.next()).floatValue()));
            }
            emptyList = arrayList;
        }
        JSONObject put2 = put.put("observedFormants", new JSONArray((Collection) emptyList)).put("processingMs", observation.getProcessingMs());
        if (this.trusted) {
            WebView webView = this.web;
            if (webView == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
                webView = null;
            }
            webView.evaluateJavascript("window.LabHost && window.LabHost.observation(" + put2 + ");", null);
        }
        submit$default(this, new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda13
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit acceptObservation$lambda$34;
                acceptObservation$lambda$34 = AnatomyBenchActivity.acceptObservation$lambda$34(AnatomyBenchActivity.this);
                return acceptObservation$lambda$34;
            }
        }, null, new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda14
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit acceptObservation$lambda$35;
                acceptObservation$lambda$35 = AnatomyBenchActivity.acceptObservation$lambda$35(AnatomyBenchActivity.this, observation, acoustic);
                return acceptObservation$lambda$35;
            }
        }, 2, null);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit acceptObservation$lambda$34(AnatomyBenchActivity anatomyBenchActivity) {
        anatomyBenchActivity.livePending.set(false);
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit acceptObservation$lambda$35(AnatomyBenchActivity anatomyBenchActivity, VocalAcousticsState vocalAcousticsState, AcousticEstimate acousticEstimate) {
        SharedLabState sharedLabState = anatomyBenchActivity.state;
        if (sharedLabState == null) {
            Intrinsics.throwUninitializedPropertyAccessException("state");
            sharedLabState = null;
        }
        sharedLabState.acceptLive(vocalAcousticsState.getTract(), acousticEstimate.getF0Hz(), Intrinsics.areEqual(vocalAcousticsState.getSource(), "shared-model-replay") ? "Model replay" : "Microphone");
        return Unit.INSTANCE;
    }

    private final void stopTone() {
        this.playWanted = false;
        AdditiveSynthEngine additiveSynthEngine = this.synth;
        AudioFocusRequest audioFocusRequest = null;
        if (additiveSynthEngine != null) {
            if (additiveSynthEngine == null) {
                Intrinsics.throwUninitializedPropertyAccessException("synth");
                additiveSynthEngine = null;
            }
            additiveSynthEngine.stop();
        }
        AudioManager audioManager = this.audioManager;
        if (audioManager != null) {
            if (audioManager == null) {
                Intrinsics.throwUninitializedPropertyAccessException("audioManager");
                audioManager = null;
            }
            AudioFocusRequest audioFocusRequest2 = this.focusRequest;
            if (audioFocusRequest2 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("focusRequest");
            } else {
                audioFocusRequest = audioFocusRequest2;
            }
            audioManager.abandonAudioFocusRequest(audioFocusRequest);
        }
        DiagnosticsRuntime.INSTANCE.updateSynthesizerActive(false);
    }

    private final void stopInput() {
        this.epoch.incrementAndGet();
        SingerCalibrationController singerCalibrationController = null;
        this.permissionEpoch = null;
        this.inputMode = "idle";
        AudioCaptureEngine audioCaptureEngine = this.capture;
        if (audioCaptureEngine != null) {
            if (audioCaptureEngine == null) {
                Intrinsics.throwUninitializedPropertyAccessException("capture");
                audioCaptureEngine = null;
            }
            audioCaptureEngine.stop();
        }
        ReplayController replayController = this.replay;
        if (replayController != null) {
            if (replayController == null) {
                Intrinsics.throwUninitializedPropertyAccessException("replay");
                replayController = null;
            }
            replayController.stop();
        }
        RealtimeDspPipeline realtimeDspPipeline = this.pipeline;
        if (realtimeDspPipeline != null) {
            if (realtimeDspPipeline == null) {
                Intrinsics.throwUninitializedPropertyAccessException("pipeline");
                realtimeDspPipeline = null;
            }
            realtimeDspPipeline.stop(false);
        }
        SingerCalibrationController singerCalibrationController2 = this.calibration;
        if (singerCalibrationController2 != null) {
            if (singerCalibrationController2 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("calibration");
                singerCalibrationController2 = null;
            }
            if (singerCalibrationController2.isActive()) {
                SingerCalibrationController singerCalibrationController3 = this.calibration;
                if (singerCalibrationController3 == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("calibration");
                } else {
                    singerCalibrationController = singerCalibrationController3;
                }
                this.calibrationText = singerCalibrationController.cancel().getPrompt();
            }
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public final void stopEverything() {
        stopTone();
        stopInput();
    }

    /* JADX WARN: Removed duplicated region for block: B:17:0x005b  */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    private final void sendStatus() {
        boolean z;
        WebView webView;
        if (this.destroyed || this.web == null || !this.trusted) {
            return;
        }
        JSONObject jSONObject = new JSONObject();
        if (this.playWanted) {
            AdditiveSynthEngine additiveSynthEngine = this.synth;
            if (additiveSynthEngine == null) {
                Intrinsics.throwUninitializedPropertyAccessException("synth");
                additiveSynthEngine = null;
            }
            if (additiveSynthEngine.isRunning()) {
                z = true;
                JSONObject put = jSONObject.put("playing", z).put("inputMode", this.inputMode).put("message", this.statusMessage).put("calibration", this.calibrationText).put("diagnosticOverlay", DiagnosticsRuntime.INSTANCE.debugOverlayEnabled(this));
                webView = this.web;
                if (webView == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("web");
                    webView = null;
                }
                webView.evaluateJavascript("window.LabHost && window.LabHost.status(" + put + ");", null);
            }
        }
        z = false;
        JSONObject put2 = jSONObject.put("playing", z).put("inputMode", this.inputMode).put("message", this.statusMessage).put("calibration", this.calibrationText).put("diagnosticOverlay", DiagnosticsRuntime.INSTANCE.debugOverlayEnabled(this));
        webView = this.web;
        if (webView == null) {
        }
        webView.evaluateJavascript("window.LabHost && window.LabHost.status(" + put2 + ");", null);
    }

    @Override // android.app.Activity
    protected void onPause() {
        WebView webView = null;
        if (this.web != null && this.trusted) {
            WebView webView2 = this.web;
            if (webView2 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
                webView2 = null;
            }
            webView2.evaluateJavascript("window.LabHost && window.LabHost.lifecycle();", null);
        }
        this.resumed = false;
        stopEverything();
        sendStatus();
        WebView webView3 = this.web;
        if (webView3 != null) {
            if (webView3 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
            } else {
                webView = webView3;
            }
            webView.onPause();
        }
        super.onPause();
    }

    @Override // android.app.Activity
    protected void onResume() {
        super.onResume();
        this.resumed = true;
        WebView webView = this.web;
        if (webView != null) {
            if (webView == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
                webView = null;
            }
            webView.onResume();
            if (this.trusted) {
                WebView webView2 = this.web;
                if (webView2 == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("web");
                    webView2 = null;
                }
                webView2.evaluateJavascript("window.LabHost && window.LabHost.lifecycle();", null);
                submit$default(this, null, null, new Function0() { // from class: org.vocaltract.pixel.AnatomyBenchActivity$$ExternalSyntheticLambda0
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit unit;
                        unit = Unit.INSTANCE;
                        return unit;
                    }
                }, 3, null);
            }
        }
    }

    @Override // android.app.Activity
    protected void onDestroy() {
        stopEverything();
        this.destroyed = true;
        this.trusted = false;
        this.executor.shutdownNow();
        AdditiveSynthEngine additiveSynthEngine = this.synth;
        WebView webView = null;
        if (additiveSynthEngine != null) {
            if (additiveSynthEngine == null) {
                Intrinsics.throwUninitializedPropertyAccessException("synth");
                additiveSynthEngine = null;
            }
            additiveSynthEngine.release();
        }
        WebView webView2 = this.web;
        if (webView2 != null) {
            if (webView2 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
                webView2 = null;
            }
            webView2.removeJavascriptInterface("NativeLab");
            WebView webView3 = this.web;
            if (webView3 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
                webView3 = null;
            }
            ViewParent parent = webView3.getParent();
            ViewGroup viewGroup = parent instanceof ViewGroup ? (ViewGroup) parent : null;
            if (viewGroup != null) {
                WebView webView4 = this.web;
                if (webView4 == null) {
                    Intrinsics.throwUninitializedPropertyAccessException("web");
                    webView4 = null;
                }
                viewGroup.removeView(webView4);
            }
            WebView webView5 = this.web;
            if (webView5 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("web");
            } else {
                webView = webView5;
            }
            webView.destroy();
        }
        super.onDestroy();
    }
}
