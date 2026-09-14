package org.vocaltract.pixel;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.DialogInterface;
import android.content.Intent;
import android.graphics.Color;
import android.graphics.Insets;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.view.View;
import android.view.WindowInsets;
import android.widget.Button;
import android.widget.CheckBox;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import android.widget.Toast;
import java.text.DateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.Iterator;
import java.util.List;
import kotlin.Metadata;
import kotlin.Result;
import kotlin.ResultKt;
import kotlin.Unit;
import kotlin.collections.CollectionsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.Intrinsics;
import kotlin.text.Regex;
import kotlin.text.StringsKt;

/* compiled from: AdminConsoleActivity.kt */
@Metadata(d1 = {"\u0000b\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\n\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0007\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010\b\n\u0002\b\u0002\u0018\u00002\u00020\u0001B\u0007¢\u0006\u0004\b\u0002\u0010\u0003J\u0012\u0010\u0012\u001a\u00020\u00132\b\u0010\u0014\u001a\u0004\u0018\u00010\u0015H\u0014J\b\u0010\u0016\u001a\u00020\u0013H\u0002J\b\u0010\u0017\u001a\u00020\u0013H\u0002J\b\u0010\u0018\u001a\u00020\u0013H\u0002J\b\u0010\u0019\u001a\u00020\u0013H\u0002J\b\u0010\u001a\u001a\u00020\u0013H\u0002J\b\u0010\u001b\u001a\u00020\u0013H\u0002J\b\u0010\u001c\u001a\u00020\u0013H\u0002J\b\u0010\u001d\u001a\u00020\u0013H\u0002J\b\u0010\u001e\u001a\u00020\u0013H\u0014J\b\u0010\u001f\u001a\u00020 H\u0002J\u0018\u0010!\u001a\u00020\u00052\u0006\u0010\"\u001a\u00020#2\u0006\u0010$\u001a\u00020%H\u0002J\u0010\u0010&\u001a\u00020\u00052\u0006\u0010\"\u001a\u00020#H\u0002J\u0010\u0010'\u001a\u00020\u00052\u0006\u0010\"\u001a\u00020#H\u0002J\u0010\u0010(\u001a\u00020\f2\u0006\u0010)\u001a\u00020#H\u0002J\u001e\u0010*\u001a\u00020\n2\u0006\u0010+\u001a\u00020#2\f\u0010,\u001a\b\u0012\u0004\u0012\u00020\u00130-H\u0002J\b\u0010.\u001a\u00020/H\u0002J\b\u00100\u001a\u00020/H\u0002J\u0010\u00101\u001a\u0002022\u0006\u00103\u001a\u000202H\u0002J\u0010\u00104\u001a\u00020#2\u0006\u00103\u001a\u00020%H\u0002R\u000e\u0010\u0004\u001a\u00020\u0005X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0005X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0007\u001a\u00020\u0005X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\b\u001a\u00020\u0005X\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\t\u001a\u00020\nX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u000b\u001a\u00020\fX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\r\u001a\u00020\fX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u000e\u001a\u00020\fX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u000f\u001a\u00020\fX\u0082.¢\u0006\u0002\n\u0000R\u000e\u0010\u0010\u001a\u00020\u0011X\u0082.¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/AdminConsoleActivity;", "Landroid/app/Activity;", "<init>", "()V", "assistantText", "Landroid/widget/TextView;", "selfTestText", "eventText", "calibrationText", "overlayButton", "Landroid/widget/Button;", "expectedF0", "Landroid/widget/EditText;", "expectedVowel", "expectedResonances", "correctionNotes", "approveTraining", "Landroid/widget/CheckBox;", "onCreate", "", "savedInstanceState", "Landroid/os/Bundle;", "createUi", "refresh", "runSelfTests", "saveCorrection", "exportBundle", "refreshEvents", "confirmClear", "updateOverlayButton", "onDestroy", "row", "Landroid/widget/LinearLayout;", "title", "textValue", "", "size", "", "section", "body", "input", "hintValue", "action", "label", "click", "Lkotlin/Function0;", "weight", "Landroid/widget/LinearLayout$LayoutParams;", "fullWidth", "dp", "", "value", "oneDecimal"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AdminConsoleActivity extends Activity {
    private CheckBox approveTraining;
    private TextView assistantText;
    private TextView calibrationText;
    private EditText correctionNotes;
    private TextView eventText;
    private EditText expectedF0;
    private EditText expectedResonances;
    private EditText expectedVowel;
    private Button overlayButton;
    private TextView selfTestText;

    @Override // android.app.Activity
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        DiagnosticsRuntime diagnosticsRuntime = DiagnosticsRuntime.INSTANCE;
        Context applicationContext = getApplicationContext();
        Intrinsics.checkNotNullExpressionValue(applicationContext, "getApplicationContext(...)");
        diagnosticsRuntime.initialize(applicationContext);
        DiagnosticsRuntime.log$default(DiagnosticsRuntime.INSTANCE, "lifecycle", "admin_opened", "Engineering console opened", null, null, 24, null);
        createUi();
        refresh();
    }

    private final void createUi() {
        Object m4constructorimpl;
        AdminConsoleActivity adminConsoleActivity = this;
        LinearLayout linearLayout = new LinearLayout(adminConsoleActivity);
        linearLayout.setOrientation(1);
        linearLayout.setPadding(dp(18), dp(16), dp(18), dp(30));
        linearLayout.setBackgroundColor(Color.rgb(244, 248, 252));
        if (Build.VERSION.SDK_INT >= 30) {
            linearLayout.setOnApplyWindowInsetsListener(new View.OnApplyWindowInsetsListener() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda2
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // android.view.View.OnApplyWindowInsetsListener
                public final WindowInsets onApplyWindowInsets(View view, WindowInsets windowInsets) {
                    WindowInsets createUi$lambda$1;
                    createUi$lambda$1 = AdminConsoleActivity.createUi$lambda$1(AdminConsoleActivity.this, view, windowInsets);
                    return createUi$lambda$1;
                }
            });
        }
        linearLayout.addView(title("Engineering Console", 27.0f));
        linearLayout.addView(body("Session " + DiagnosticsRuntime.INSTANCE.getSessionId() + "\nDerived metrics only • no raw audio • no automatic upload • no automatic model mutation"));
        linearLayout.addView(section("Active 3D atlas"));
        try {
            Result.Companion companion = Result.INSTANCE;
            AdminConsoleActivity adminConsoleActivity2 = this;
            TractLumenAsset load = TractLumenAsset.INSTANCE.load(this);
            m4constructorimpl = Result.m4constructorimpl(load.getModelId() + "\n" + load.getProvenance() + "\nSource locks: " + CollectionsKt.joinToString$default(load.getSourceSha256(), null, null, null, 0, null, new Function1() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda3
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function1
                public final Object invoke(Object obj) {
                    CharSequence createUi$lambda$3$lambda$2;
                    createUi$lambda$3$lambda$2 = AdminConsoleActivity.createUi$lambda$3$lambda$2((String) obj);
                    return createUi$lambda$3$lambda$2;
                }
            }, 31, null));
        } catch (Throwable th) {
            Result.Companion companion2 = Result.INSTANCE;
            m4constructorimpl = Result.m4constructorimpl(ResultKt.createFailure(th));
        }
        Throwable m7exceptionOrNullimpl = Result.m7exceptionOrNullimpl(m4constructorimpl);
        if (m7exceptionOrNullimpl != null) {
            String message = m7exceptionOrNullimpl.getMessage();
            if (message == null) {
                message = m7exceptionOrNullimpl.getClass().getSimpleName();
            }
            m4constructorimpl = "Atlas unavailable: " + message;
        }
        linearLayout.addView(body((String) m4constructorimpl));
        linearLayout.addView(section("Offline diagnostic assistant"));
        TextView body = body("Analyzing latest metrics…");
        this.assistantText = body;
        TextView textView = null;
        if (body == null) {
            Intrinsics.throwUninitializedPropertyAccessException("assistantText");
            body = null;
        }
        linearLayout.addView(body);
        LinearLayout row = row();
        row.addView(action("Refresh advice", new Function0() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda4
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit createUi$lambda$7$lambda$5;
                createUi$lambda$7$lambda$5 = AdminConsoleActivity.createUi$lambda$7$lambda$5(AdminConsoleActivity.this);
                return createUi$lambda$7$lambda$5;
            }
        }), weight());
        row.addView(action("Run self-test", new Function0() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda5
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit createUi$lambda$7$lambda$6;
                createUi$lambda$7$lambda$6 = AdminConsoleActivity.createUi$lambda$7$lambda$6(AdminConsoleActivity.this);
                return createUi$lambda$7$lambda$6;
            }
        }), weight());
        linearLayout.addView(row);
        Button action = action("Enable live overlay", new Function0() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda6
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit createUi$lambda$8;
                createUi$lambda$8 = AdminConsoleActivity.createUi$lambda$8(AdminConsoleActivity.this);
                return createUi$lambda$8;
            }
        });
        this.overlayButton = action;
        if (action == null) {
            Intrinsics.throwUninitializedPropertyAccessException("overlayButton");
            action = null;
        }
        linearLayout.addView(action, fullWidth());
        linearLayout.addView(section("Singer calibration"));
        TextView body2 = body("No completed calibration in this session");
        this.calibrationText = body2;
        if (body2 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("calibrationText");
            body2 = null;
        }
        linearLayout.addView(body2);
        linearLayout.addView(section("Self-test results"));
        TextView body3 = body("Not run in this session");
        this.selfTestText = body3;
        if (body3 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("selfTestText");
            body3 = null;
        }
        linearLayout.addView(body3);
        linearLayout.addView(section("Propose a refinement"));
        linearLayout.addView(body("Corrections are appended as evidence. They never alter the active model. Only enter a target you know independently, such as a reference-tone F0."));
        this.expectedF0 = input("Expected F0 Hz (optional)");
        this.expectedVowel = input("Expected vowel label (optional)");
        this.expectedResonances = input("Expected R1–R6 Hz, comma separated (optional)");
        this.correctionNotes = input("Observation, environment, or failure notes");
        CheckBox checkBox = new CheckBox(adminConsoleActivity);
        checkBox.setText("Approved for later reviewed training");
        checkBox.setChecked(false);
        checkBox.setTextColor(Color.rgb(35, 55, 70));
        this.approveTraining = checkBox;
        EditText editText = this.expectedF0;
        if (editText == null) {
            Intrinsics.throwUninitializedPropertyAccessException("expectedF0");
            editText = null;
        }
        linearLayout.addView(editText);
        EditText editText2 = this.expectedVowel;
        if (editText2 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("expectedVowel");
            editText2 = null;
        }
        linearLayout.addView(editText2);
        EditText editText3 = this.expectedResonances;
        if (editText3 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("expectedResonances");
            editText3 = null;
        }
        linearLayout.addView(editText3);
        EditText editText4 = this.correctionNotes;
        if (editText4 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("correctionNotes");
            editText4 = null;
        }
        linearLayout.addView(editText4);
        CheckBox checkBox2 = this.approveTraining;
        if (checkBox2 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("approveTraining");
            checkBox2 = null;
        }
        linearLayout.addView(checkBox2);
        linearLayout.addView(action("Save proposed correction", new Function0() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda7
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit createUi$lambda$10;
                createUi$lambda$10 = AdminConsoleActivity.createUi$lambda$10(AdminConsoleActivity.this);
                return createUi$lambda$10;
            }
        }), fullWidth());
        linearLayout.addView(section("Progress and review bundle"));
        linearLayout.addView(body("Export writes a JSON bundle to Downloads/VocalTractLab and opens Android sharing so it can be attached directly. This app has no Internet permission."));
        linearLayout.addView(action("Export and share AI review bundle", new Function0() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda8
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit createUi$lambda$11;
                createUi$lambda$11 = AdminConsoleActivity.createUi$lambda$11(AdminConsoleActivity.this);
                return createUi$lambda$11;
            }
        }), fullWidth());
        linearLayout.addView(section("Recent session events"));
        TextView body4 = body("No events");
        this.eventText = body4;
        if (body4 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("eventText");
        } else {
            textView = body4;
        }
        linearLayout.addView(textView);
        LinearLayout row2 = row();
        row2.addView(action("Refresh log", new Function0() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda9
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit createUi$lambda$14$lambda$12;
                createUi$lambda$14$lambda$12 = AdminConsoleActivity.createUi$lambda$14$lambda$12(AdminConsoleActivity.this);
                return createUi$lambda$14$lambda$12;
            }
        }), weight());
        row2.addView(action("Clear local logs", new Function0() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda10
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit createUi$lambda$14$lambda$13;
                createUi$lambda$14$lambda$13 = AdminConsoleActivity.createUi$lambda$14$lambda$13(AdminConsoleActivity.this);
                return createUi$lambda$14$lambda$13;
            }
        }), weight());
        linearLayout.addView(row2);
        linearLayout.addView(action("Return to vocal tract", new Function0() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda11
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function0
            public final Object invoke() {
                Unit createUi$lambda$15;
                createUi$lambda$15 = AdminConsoleActivity.createUi$lambda$15(AdminConsoleActivity.this);
                return createUi$lambda$15;
            }
        }), fullWidth());
        ScrollView scrollView = new ScrollView(adminConsoleActivity);
        scrollView.addView(linearLayout);
        setContentView(scrollView);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final WindowInsets createUi$lambda$1(AdminConsoleActivity adminConsoleActivity, View view, WindowInsets insets) {
        Intrinsics.checkNotNullParameter(view, "view");
        Intrinsics.checkNotNullParameter(insets, "insets");
        Insets insets2 = insets.getInsets(WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout() | WindowInsets.Type.ime());
        Intrinsics.checkNotNullExpressionValue(insets2, "getInsets(...)");
        view.setPadding(adminConsoleActivity.dp(18) + insets2.left, adminConsoleActivity.dp(16) + insets2.top, adminConsoleActivity.dp(18) + insets2.right, adminConsoleActivity.dp(30) + insets2.bottom);
        return WindowInsets.CONSUMED;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final CharSequence createUi$lambda$3$lambda$2(String it) {
        Intrinsics.checkNotNullParameter(it, "it");
        return StringsKt.take(it, 12);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit createUi$lambda$7$lambda$5(AdminConsoleActivity adminConsoleActivity) {
        adminConsoleActivity.refresh();
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit createUi$lambda$7$lambda$6(AdminConsoleActivity adminConsoleActivity) {
        adminConsoleActivity.runSelfTests();
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit createUi$lambda$8(AdminConsoleActivity adminConsoleActivity) {
        DiagnosticsRuntime.INSTANCE.setDebugOverlayEnabled(adminConsoleActivity, !DiagnosticsRuntime.INSTANCE.debugOverlayEnabled(r1));
        adminConsoleActivity.updateOverlayButton();
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit createUi$lambda$10(AdminConsoleActivity adminConsoleActivity) {
        adminConsoleActivity.saveCorrection();
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit createUi$lambda$11(AdminConsoleActivity adminConsoleActivity) {
        adminConsoleActivity.exportBundle();
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit createUi$lambda$14$lambda$12(AdminConsoleActivity adminConsoleActivity) {
        adminConsoleActivity.refreshEvents();
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit createUi$lambda$14$lambda$13(AdminConsoleActivity adminConsoleActivity) {
        adminConsoleActivity.confirmClear();
        return Unit.INSTANCE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit createUi$lambda$15(AdminConsoleActivity adminConsoleActivity) {
        adminConsoleActivity.finish();
        return Unit.INSTANCE;
    }

    private final void refresh() {
        String joinToString$default;
        TextView textView;
        String str;
        DiagnosticMetrics currentMetrics = DiagnosticsRuntime.INSTANCE.currentMetrics();
        DiagnosticAssessment currentAssessment = DiagnosticsRuntime.INSTANCE.currentAssessment();
        if (currentAssessment.getFindings().isEmpty()) {
            joinToString$default = "No current findings.";
        } else {
            joinToString$default = CollectionsKt.joinToString$default(currentAssessment.getFindings(), "\n\n", null, null, 0, null, new Function1() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda1
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function1
                public final Object invoke(Object obj) {
                    CharSequence refresh$lambda$17;
                    refresh$lambda$17 = AdminConsoleActivity.refresh$lambda$17((DiagnosticFinding) obj);
                    return refresh$lambda$17;
                }
            }, 30, null);
        }
        TextView textView2 = this.assistantText;
        if (textView2 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("assistantText");
            textView2 = null;
        }
        TextView textView3 = textView2;
        textView3.setText("Provider: " + currentAssessment.getProviderId() + " (offline, deterministic)\nHealth score: " + currentAssessment.getHealthScore() + "/100\n" + currentAssessment.getSummary() + "\n\nLatest: source=" + currentMetrics.getSource() + ", DSP=" + oneDecimal(currentMetrics.getProcessingMs()) + " ms, drops=" + currentMetrics.getDroppedFrames() + ", renderer=" + currentMetrics.getRendererMode() + ", tract confidence=" + ((int) (currentMetrics.getTractConfidence() * 100)) + "%\nSNR=" + oneDecimal(currentMetrics.getSnrDb()) + " dB, noise=" + currentMetrics.getNoiseState() + ", pitch=" + currentMetrics.getPitchDecision() + ", abstained=" + currentMetrics.getPosteriorAbstained() + "\n\n" + joinToString$default);
        TextView textView4 = this.calibrationText;
        if (textView4 == null) {
            Intrinsics.throwUninitializedPropertyAccessException("calibrationText");
            textView = null;
        } else {
            textView = textView4;
        }
        SingerCalibrationReport currentCalibrationReport = DiagnosticsRuntime.INSTANCE.currentCalibrationReport();
        if (currentCalibrationReport != null) {
            int size = currentCalibrationReport.getSteps().size();
            Iterator<T> it = currentCalibrationReport.getSteps().iterator();
            int i = 0;
            while (it.hasNext()) {
                i += ((CalibrationStepSummary) it.next()).getFrames();
            }
            String str2 = "Completed " + size + " derived-only steps • " + i + " frames • no raw audio • model unchanged";
            if (str2 != null) {
                str = str2;
                textView.setText(str);
                updateOverlayButton();
                refreshEvents();
            }
        }
        textView.setText(str);
        updateOverlayButton();
        refreshEvents();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final CharSequence refresh$lambda$17(DiagnosticFinding finding) {
        Intrinsics.checkNotNullParameter(finding, "finding");
        return finding.getSeverity() + ": " + finding.getTitle() + "\nEvidence: " + finding.getEvidence() + "\nAction: " + finding.getRecommendedAction();
    }

    private final void runSelfTests() {
        List<SelfTestResult> runSelfTests = DiagnosticsRuntime.INSTANCE.runSelfTests(this);
        TextView textView = this.selfTestText;
        if (textView == null) {
            Intrinsics.throwUninitializedPropertyAccessException("selfTestText");
            textView = null;
        }
        textView.setText(CollectionsKt.joinToString$default(runSelfTests, "\n", null, null, 0, null, new Function1() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda14
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                CharSequence runSelfTests$lambda$20;
                runSelfTests$lambda$20 = AdminConsoleActivity.runSelfTests$lambda$20((SelfTestResult) obj);
                return runSelfTests$lambda$20;
            }
        }, 30, null));
        refresh();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final CharSequence runSelfTests$lambda$20(SelfTestResult result) {
        Intrinsics.checkNotNullParameter(result, "result");
        return (result.getPassed() ? "PASS" : "CHECK") + "  " + result.getCode() + ": " + result.getMessage();
    }

    /* JADX DEBUG: Don't trust debug lines info. Repeating lines: [168=4] */
    /* JADX WARN: Removed duplicated region for block: B:41:0x00cb A[Catch: RuntimeException -> 0x0186, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:45:0x00da A[Catch: RuntimeException -> 0x0186, TRY_ENTER, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:49:0x00ec A[Catch: RuntimeException -> 0x0186, TRY_ENTER, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:53:0x00fe A[Catch: RuntimeException -> 0x0186, TRY_ENTER, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:56:0x0112 A[Catch: RuntimeException -> 0x0186, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:59:0x0121 A[Catch: RuntimeException -> 0x0186, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:62:0x0130 A[Catch: RuntimeException -> 0x0186, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:65:0x013f A[Catch: RuntimeException -> 0x0186, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:68:0x014e A[Catch: RuntimeException -> 0x0186, TryCatch #0 {RuntimeException -> 0x0186, blocks: (B:3:0x0003, B:6:0x000a, B:7:0x000e, B:11:0x002d, B:12:0x0038, B:15:0x003e, B:16:0x0042, B:21:0x0062, B:23:0x0071, B:24:0x007e, B:26:0x0084, B:29:0x0094, B:34:0x0098, B:35:0x00ad, B:37:0x00b3, B:39:0x00c5, B:41:0x00cb, B:42:0x00cf, B:45:0x00da, B:46:0x00de, B:49:0x00ec, B:50:0x00f0, B:53:0x00fe, B:54:0x0102, B:56:0x0112, B:57:0x0116, B:59:0x0121, B:60:0x0125, B:62:0x0130, B:63:0x0134, B:65:0x013f, B:66:0x0143, B:68:0x014e, B:69:0x0153), top: B:2:0x0003 }] */
    /* JADX WARN: Removed duplicated region for block: B:73:0x0152  */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    private final void saveCorrection() {
        ArrayList arrayList;
        EditText editText;
        EditText editText2;
        CheckBox checkBox;
        EditText editText3;
        EditText editText4;
        EditText editText5;
        EditText editText6;
        CheckBox checkBox2;
        try {
            EditText editText7 = this.expectedF0;
            CheckBox checkBox3 = null;
            if (editText7 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("expectedF0");
                editText7 = null;
            }
            String obj = StringsKt.trim((CharSequence) editText7.getText().toString()).toString();
            if (obj.length() <= 0) {
                obj = null;
            }
            Float valueOf = obj != null ? Float.valueOf(Float.parseFloat(obj)) : null;
            EditText editText8 = this.expectedResonances;
            if (editText8 == null) {
                Intrinsics.throwUninitializedPropertyAccessException("expectedResonances");
                editText8 = null;
            }
            String obj2 = StringsKt.trim((CharSequence) editText8.getText().toString()).toString();
            if (obj2.length() <= 0) {
                obj2 = null;
            }
            if (obj2 != null) {
                List<String> split = new Regex("[,\\s]+").split(obj2, 0);
                if (split != null) {
                    ArrayList arrayList2 = new ArrayList();
                    for (Object obj3 : split) {
                        if (!StringsKt.isBlank((String) obj3)) {
                            arrayList2.add(obj3);
                        }
                    }
                    ArrayList arrayList3 = arrayList2;
                    ArrayList arrayList4 = new ArrayList(CollectionsKt.collectionSizeOrDefault(arrayList3, 10));
                    Iterator it = arrayList3.iterator();
                    while (it.hasNext()) {
                        arrayList4.add(Float.valueOf(Float.parseFloat((String) it.next())));
                    }
                    arrayList = arrayList4;
                    if (arrayList == null) {
                        arrayList = CollectionsKt.emptyList();
                    }
                    List list = arrayList;
                    DiagnosticsRuntime diagnosticsRuntime = DiagnosticsRuntime.INSTANCE;
                    editText = this.expectedVowel;
                    if (editText == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("expectedVowel");
                        editText = null;
                    }
                    String obj4 = editText.getText().toString();
                    editText2 = this.correctionNotes;
                    if (editText2 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("correctionNotes");
                        editText2 = null;
                    }
                    String obj5 = editText2.getText().toString();
                    checkBox = this.approveTraining;
                    if (checkBox == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("approveTraining");
                        checkBox = null;
                    }
                    RefinementRecord recordRefinement = diagnosticsRuntime.recordRefinement(new RefinementDraft(valueOf, obj4, list, obj5, checkBox.isChecked()));
                    editText3 = this.expectedF0;
                    if (editText3 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("expectedF0");
                        editText3 = null;
                    }
                    editText3.getText().clear();
                    editText4 = this.expectedVowel;
                    if (editText4 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("expectedVowel");
                        editText4 = null;
                    }
                    editText4.getText().clear();
                    editText5 = this.expectedResonances;
                    if (editText5 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("expectedResonances");
                        editText5 = null;
                    }
                    editText5.getText().clear();
                    editText6 = this.correctionNotes;
                    if (editText6 == null) {
                        Intrinsics.throwUninitializedPropertyAccessException("correctionNotes");
                        editText6 = null;
                    }
                    editText6.getText().clear();
                    checkBox2 = this.approveTraining;
                    if (checkBox2 != null) {
                        Intrinsics.throwUninitializedPropertyAccessException("approveTraining");
                    } else {
                        checkBox3 = checkBox2;
                    }
                    checkBox3.setChecked(false);
                    Toast.makeText(this, "Saved proposal " + StringsKt.take(recordRefinement.getRecordId(), 8) + "; model unchanged", 1).show();
                    refresh();
                }
            }
            arrayList = null;
            if (arrayList == null) {
            }
            List list2 = arrayList;
            DiagnosticsRuntime diagnosticsRuntime2 = DiagnosticsRuntime.INSTANCE;
            editText = this.expectedVowel;
            if (editText == null) {
            }
            String obj42 = editText.getText().toString();
            editText2 = this.correctionNotes;
            if (editText2 == null) {
            }
            String obj52 = editText2.getText().toString();
            checkBox = this.approveTraining;
            if (checkBox == null) {
            }
            RefinementRecord recordRefinement2 = diagnosticsRuntime2.recordRefinement(new RefinementDraft(valueOf, obj42, list2, obj52, checkBox.isChecked()));
            editText3 = this.expectedF0;
            if (editText3 == null) {
            }
            editText3.getText().clear();
            editText4 = this.expectedVowel;
            if (editText4 == null) {
            }
            editText4.getText().clear();
            editText5 = this.expectedResonances;
            if (editText5 == null) {
            }
            editText5.getText().clear();
            editText6 = this.correctionNotes;
            if (editText6 == null) {
            }
            editText6.getText().clear();
            checkBox2 = this.approveTraining;
            if (checkBox2 != null) {
            }
            checkBox3.setChecked(false);
            Toast.makeText(this, "Saved proposal " + StringsKt.take(recordRefinement2.getRecordId(), 8) + "; model unchanged", 1).show();
            refresh();
        } catch (RuntimeException e) {
            AdminConsoleActivity adminConsoleActivity = this;
            String message = e.getMessage();
            if (message == null) {
                message = "Invalid correction";
            }
            Toast.makeText(adminConsoleActivity, message, 1).show();
        }
    }

    /* JADX DEBUG: Class process forced to load method for inline: org.vocaltract.pixel.DiagnosticsRuntime.log$default(org.vocaltract.pixel.DiagnosticsRuntime, java.lang.String, java.lang.String, java.lang.String, java.util.Map, org.vocaltract.pixel.DiagnosticSeverity, int, java.lang.Object):void */
    private final void exportBundle() {
        try {
            DiagnosticExport exportBundle = DiagnosticsRuntime.INSTANCE.exportBundle();
            Intent intent = new Intent("android.intent.action.SEND");
            intent.setType("application/json");
            intent.putExtra("android.intent.extra.STREAM", Uri.parse(exportBundle.getUri()));
            intent.putExtra("android.intent.extra.SUBJECT", "Vocal Tract Lab diagnostic bundle");
            intent.addFlags(1);
            startActivity(Intent.createChooser(intent, "Share diagnostic bundle"));
            Toast.makeText(this, "Saved " + exportBundle.getDisplayName() + " to Downloads/VocalTractLab and opened sharing", 1).show();
            refreshEvents();
        } catch (Throwable th) {
            DiagnosticsRuntime diagnosticsRuntime = DiagnosticsRuntime.INSTANCE;
            String message = th.getMessage();
            if (message == null) {
                message = th.getClass().getSimpleName();
            }
            String str = message;
            Intrinsics.checkNotNull(str);
            DiagnosticsRuntime.log$default(diagnosticsRuntime, "export", "bundle_export_failed", str, null, DiagnosticSeverity.ERROR, 8, null);
            Toast.makeText(this, "Export failed: " + th.getMessage(), 1).show();
        }
    }

    private final void refreshEvents() {
        String joinToString$default;
        final DateFormat timeInstance = DateFormat.getTimeInstance(2);
        List<DiagnosticEvent> recentEvents = DiagnosticsRuntime.INSTANCE.recentEvents(30);
        TextView textView = this.eventText;
        if (textView == null) {
            Intrinsics.throwUninitializedPropertyAccessException("eventText");
            textView = null;
        }
        if (!recentEvents.isEmpty()) {
            joinToString$default = CollectionsKt.joinToString$default(CollectionsKt.asReversed(recentEvents), "\n", null, null, 0, null, new Function1() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda12
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function1
                public final Object invoke(Object obj) {
                    CharSequence refreshEvents$lambda$25;
                    refreshEvents$lambda$25 = AdminConsoleActivity.refreshEvents$lambda$25(timeInstance, (DiagnosticEvent) obj);
                    return refreshEvents$lambda$25;
                }
            }, 30, null);
        }
        textView.setText(joinToString$default);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final CharSequence refreshEvents$lambda$25(DateFormat dateFormat, DiagnosticEvent event) {
        Intrinsics.checkNotNullParameter(event, "event");
        return dateFormat.format(new Date(event.getTimestampMillis())) + " " + event.getSeverity() + " " + event.getCategory() + "/" + event.getCode() + ": " + event.getMessage();
    }

    private final void confirmClear() {
        new AlertDialog.Builder(this).setTitle("Clear local diagnostics?").setMessage("This removes stored events, corrections, and self-test history from the app. Export first if needed.").setNegativeButton("Cancel", (DialogInterface.OnClickListener) null).setPositiveButton("Clear", new DialogInterface.OnClickListener() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // android.content.DialogInterface.OnClickListener
            public final void onClick(DialogInterface dialogInterface, int i) {
                AdminConsoleActivity.confirmClear$lambda$26(AdminConsoleActivity.this, dialogInterface, i);
            }
        }).show();
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final void confirmClear$lambda$26(AdminConsoleActivity adminConsoleActivity, DialogInterface dialogInterface, int i) {
        DiagnosticsRuntime.INSTANCE.clearLogs();
        adminConsoleActivity.refresh();
    }

    private final void updateOverlayButton() {
        String str;
        Button button = this.overlayButton;
        if (button == null) {
            Intrinsics.throwUninitializedPropertyAccessException("overlayButton");
            button = null;
        }
        if (DiagnosticsRuntime.INSTANCE.debugOverlayEnabled(this)) {
        }
        button.setText(str);
    }

    @Override // android.app.Activity
    protected void onDestroy() {
        DiagnosticsRuntime.log$default(DiagnosticsRuntime.INSTANCE, "lifecycle", "admin_closed", "Engineering console closed", null, null, 24, null);
        super.onDestroy();
    }

    private final LinearLayout row() {
        LinearLayout linearLayout = new LinearLayout(this);
        linearLayout.setOrientation(0);
        linearLayout.setGravity(1);
        return linearLayout;
    }

    private final TextView title(String textValue, float size) {
        TextView textView = new TextView(this);
        textView.setText(textValue);
        textView.setTextSize(size);
        textView.setTextColor(Color.rgb(20, 48, 68));
        textView.setPadding(0, 0, 0, dp(8));
        return textView;
    }

    private final TextView section(String textValue) {
        TextView title = title(textValue, 19.0f);
        title.setPadding(0, dp(18), 0, dp(6));
        return title;
    }

    private final TextView body(String textValue) {
        TextView textView = new TextView(this);
        textView.setText(textValue);
        textView.setTextSize(14.0f);
        textView.setTextColor(Color.rgb(38, 60, 72));
        textView.setPadding(0, dp(4), 0, dp(6));
        return textView;
    }

    private final EditText input(String hintValue) {
        EditText editText = new EditText(this);
        editText.setHint(hintValue);
        editText.setTextSize(14.0f);
        editText.setSingleLine(false);
        editText.setMaxLines(4);
        return editText;
    }

    private final Button action(String label, final Function0<Unit> click) {
        Button button = new Button(this);
        button.setText(label);
        button.setOnClickListener(new View.OnClickListener() { // from class: org.vocaltract.pixel.AdminConsoleActivity$$ExternalSyntheticLambda13
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // android.view.View.OnClickListener
            public final void onClick(View view) {
                Function0.this.invoke();
            }
        });
        return button;
    }

    private final LinearLayout.LayoutParams weight() {
        return new LinearLayout.LayoutParams(0, -2, 1.0f);
    }

    private final LinearLayout.LayoutParams fullWidth() {
        return new LinearLayout.LayoutParams(-1, -2);
    }

    private final int dp(int value) {
        return (int) (value * getResources().getDisplayMetrics().density);
    }

    private final String oneDecimal(float value) {
        return String.valueOf(((int) (value * 10.0f)) / 10.0f);
    }
}
