package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.Iterator;
import java.util.List;
import kotlin.Metadata;
import kotlin.NoWhenBranchMatchedException;
import kotlin.collections.CollectionsKt;
import kotlin.comparisons.ComparisonsKt;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u0000$\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\u000e\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\t\u0018\u00002\u00020\u0001B\u0007¢\u0006\u0004\b\u0002\u0010\u0003J\u0018\u0010\b\u001a\u00020\t2\u0006\u0010\n\u001a\u00020\u000b2\u0006\u0010\f\u001a\u00020\rH\u0016R\u0014\u0010\u0004\u001a\u00020\u0005X\u0096D¢\u0006\b\n\u0000\u001a\u0004\b\u0006\u0010\u0007"}, d2 = {"Lorg/vocaltract/pixel/OfflineDiagnosticAssistant;", "Lorg/vocaltract/pixel/DiagnosticAssistant;", "<init>", "()V", "providerId", "", "getProviderId", "()Ljava/lang/String;", "assess", "Lorg/vocaltract/pixel/DiagnosticAssessment;", "metrics", "Lorg/vocaltract/pixel/DiagnosticMetrics;", "nowMillis", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class OfflineDiagnosticAssistant implements DiagnosticAssistant {
    private final String providerId = "offline_diagnostic_assistant_v1";

    /* compiled from: DiagnosticsCore.kt */
    @Metadata(k = 3, mv = {2, 0, 0}, xi = 48)
    public /* synthetic */ class WhenMappings {
        public static final /* synthetic */ int[] $EnumSwitchMapping$0;

        static {
            int[] iArr = new int[DiagnosticSeverity.values().length];
            try {
                iArr[DiagnosticSeverity.ERROR.ordinal()] = 1;
            } catch (NoSuchFieldError unused) {
            }
            try {
                iArr[DiagnosticSeverity.WARNING.ordinal()] = 2;
            } catch (NoSuchFieldError unused2) {
            }
            try {
                iArr[DiagnosticSeverity.INFO.ordinal()] = 3;
            } catch (NoSuchFieldError unused3) {
            }
            $EnumSwitchMapping$0 = iArr;
        }
    }

    @Override // org.vocaltract.pixel.DiagnosticAssistant
    public String getProviderId() {
        return this.providerId;
    }

    /* JADX WARN: Code restructure failed: missing block: B:28:0x0176, code lost:
    
        r1 = org.vocaltract.pixel.DiagnosticsCoreKt.oneDecimal(r1.floatValue());
     */
    /* JADX WARN: Code restructure failed: missing block: B:32:0x0187, code lost:
    
        r4 = org.vocaltract.pixel.DiagnosticsCoreKt.oneDecimal(r4.floatValue());
     */
    @Override // org.vocaltract.pixel.DiagnosticAssistant
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public DiagnosticAssessment assess(DiagnosticMetrics metrics, long nowMillis) {
        String oneDecimal;
        String oneDecimal2;
        int percent;
        int percent2;
        String str;
        int i;
        int percent3;
        int percent4;
        String oneDecimal3;
        String oneDecimal4;
        int percent5;
        int percent6;
        String oneDecimal5;
        String str2;
        String oneDecimal6;
        String oneDecimal7;
        String oneDecimal8;
        Intrinsics.checkNotNullParameter(metrics, "metrics");
        ArrayList arrayList = new ArrayList();
        if (Intrinsics.areEqual(metrics.getRendererMode(), "2d_fallback")) {
            assess$add(arrayList, "renderer_fallback", DiagnosticSeverity.ERROR, "3D renderer is in fallback mode", "renderer_mode=2d_fallback", "Record the status message, restart once, and export this bundle for shader/context review.", 0.99f);
        }
        if (metrics.getProcessingMs() > metrics.getFrameBudgetMs()) {
            DiagnosticSeverity diagnosticSeverity = DiagnosticSeverity.ERROR;
            oneDecimal7 = DiagnosticsCoreKt.oneDecimal(metrics.getProcessingMs());
            oneDecimal8 = DiagnosticsCoreKt.oneDecimal(metrics.getFrameBudgetMs());
            assess$add(arrayList, "frame_deadline_missed", diagnosticSeverity, "DSP exceeded its frame deadline", oneDecimal7 + " ms > " + oneDecimal8 + " ms", "Stop other audio modes, repeat the fixture, and inspect sustained device timing and thermals.", 0.98f);
        } else if (metrics.getProcessingMs() > metrics.getFrameBudgetMs() * 0.5f) {
            DiagnosticSeverity diagnosticSeverity2 = DiagnosticSeverity.WARNING;
            oneDecimal = DiagnosticsCoreKt.oneDecimal(metrics.getProcessingMs());
            oneDecimal2 = DiagnosticsCoreKt.oneDecimal(metrics.getFrameBudgetMs());
            assess$add(arrayList, "frame_budget_pressure", diagnosticSeverity2, "DSP is using more than half its frame budget", oneDecimal + " of " + oneDecimal2 + " ms", "Run a sustained test and watch queue drops before accepting this device configuration.", 0.92f);
        }
        if (metrics.getDroppedFrames() > 0) {
            assess$add(arrayList, "queue_drops", metrics.getDroppedFrames() >= 10 ? DiagnosticSeverity.ERROR : DiagnosticSeverity.WARNING, "Audio frames were dropped", "dropped_frames=" + metrics.getDroppedFrames(), "Close competing audio workloads and compare drops against processing time and thermal state.", 0.97f);
        }
        if (Intrinsics.areEqual(metrics.getSource(), "live") && !metrics.getMicrophoneGranted()) {
            assess$add(arrayList, "microphone_permission", DiagnosticSeverity.ERROR, "Live mode lacks microphone permission", "permission=denied", "Grant microphone access in Android settings or use the offline demo.", 1.0f);
        }
        if (metrics.getVoiced() && metrics.getF0Hz() == null) {
            assess$add(arrayList, "voiced_without_f0", DiagnosticSeverity.ERROR, "Voicing and F0 disagree", "voiced=true, f0=null", "Capture a diagnostic fixture; this is an internal estimator-consistency error.", 1.0f);
        } else if (metrics.getVoiced() && metrics.getF0Confidence() < 0.55f) {
            DiagnosticSeverity diagnosticSeverity3 = DiagnosticSeverity.WARNING;
            percent = DiagnosticsCoreKt.percent(metrics.getF0Confidence());
            assess$add(arrayList, "low_f0_confidence", diagnosticSeverity3, "Pitch estimate is uncertain", "f0_confidence=" + percent + "%", "Reduce background sound, move closer, sustain one pitch, and save the known F0 as a correction.", 0.9f);
        }
        if (metrics.getPitchRejected()) {
            DiagnosticSeverity diagnosticSeverity4 = DiagnosticSeverity.WARNING;
            Float rawF0Hz = metrics.getRawF0Hz();
            String str3 = "null";
            if (rawF0Hz == null || str2 == null) {
                str2 = "null";
            }
            Float f0Hz = metrics.getF0Hz();
            if (f0Hz != null && oneDecimal6 != null) {
                str3 = oneDecimal6;
            }
            assess$add(arrayList, "pitch_continuity_intervention", diagnosticSeverity4, "A pitch discontinuity was corrected or held", "raw_f0=" + str2 + " Hz; accepted_f0=" + str3 + " Hz; decision=" + metrics.getPitchDecision(), "Check the accepted pitch against a reference tone before using this frame as correction evidence.", 0.96f);
        }
        if (metrics.getVoiced() && metrics.getSnrDb() < 6.0f) {
            DiagnosticSeverity diagnosticSeverity5 = DiagnosticSeverity.WARNING;
            oneDecimal5 = DiagnosticsCoreKt.oneDecimal(metrics.getSnrDb());
            assess$add(arrayList, "low_signal_to_noise", diagnosticSeverity5, "Singer-to-background ratio is low", "snr=" + oneDecimal5 + " dB; noise_state=" + metrics.getNoiseState(), "Repeat the background calibration with the fan running, then sing closer to the phone.", 0.93f);
        }
        if (metrics.getBackgroundChanged()) {
            DiagnosticSeverity diagnosticSeverity6 = DiagnosticSeverity.WARNING;
            String noiseState = metrics.getNoiseState();
            percent6 = DiagnosticsCoreKt.percent(metrics.getNoiseConfidence());
            assess$add(arrayList, "background_changed", diagnosticSeverity6, "The stationary background changed", "noise_state=" + noiseState + "; confidence=" + percent6 + "%", "Pause singing and relearn the background, especially after a fan speed or position change.", 0.95f);
        }
        if (metrics.getPosteriorAbstained()) {
            DiagnosticSeverity diagnosticSeverity7 = DiagnosticSeverity.WARNING;
            String abstentionReason = metrics.getAbstentionReason();
            percent5 = DiagnosticsCoreKt.percent(metrics.getTractConfidence());
            assess$add(arrayList, "posterior_abstained", diagnosticSeverity7, "The 3D posterior abstained", "reason=" + abstentionReason + "; tract_confidence=" + percent5 + "%", "Treat the mean-shape display as uncertainty, not as the singer's observed anatomy.", 0.99f);
        }
        Float f0Hz2 = metrics.getF0Hz();
        if (f0Hz2 != null && (f0Hz2.floatValue() < 50.0f || f0Hz2.floatValue() > 1500.0f)) {
            DiagnosticSeverity diagnosticSeverity8 = DiagnosticSeverity.WARNING;
            oneDecimal4 = DiagnosticsCoreKt.oneDecimal(f0Hz2.floatValue());
            assess$add(arrayList, "unusual_f0", diagnosticSeverity8, "Pitch is outside the normal analysis range", "f0=" + oneDecimal4 + " Hz", "Confirm the octave against a reference tone before accepting or correcting the estimate.", 0.8f);
        }
        if (metrics.getMeanFormantStdHz() > 250.0f) {
            DiagnosticSeverity diagnosticSeverity9 = DiagnosticSeverity.WARNING;
            oneDecimal3 = DiagnosticsCoreKt.oneDecimal(metrics.getMeanFormantStdHz());
            assess$add(arrayList, "uncertain_resonances", diagnosticSeverity9, "Resonance estimates are broad", "mean_std=" + oneDecimal3 + " Hz", "Hold a steadier vowel and separate room/device calibration from the singer estimate.", 0.88f);
        }
        if (metrics.getTractConfidence() < 0.15f) {
            DiagnosticSeverity diagnosticSeverity10 = DiagnosticSeverity.ERROR;
            percent4 = DiagnosticsCoreKt.percent(metrics.getTractConfidence());
            assess$add(arrayList, "very_low_tract_confidence", diagnosticSeverity10, "Tract estimate should not be trusted", "tract_confidence=" + percent4 + "%", "Treat the mesh as abstained; do not use this frame as refinement data.", 0.98f);
        } else if (metrics.getTractConfidence() < 0.35f) {
            DiagnosticSeverity diagnosticSeverity11 = DiagnosticSeverity.WARNING;
            percent2 = DiagnosticsCoreKt.percent(metrics.getTractConfidence());
            assess$add(arrayList, "low_tract_confidence", diagnosticSeverity11, "Tract posterior is weakly constrained", "tract_confidence=" + percent2 + "%", "Repeat with a sustained vowel and only save a correction if the target is independently known.", 0.94f);
        }
        if (metrics.getRelativeAreaStd() > 0.65f) {
            DiagnosticSeverity diagnosticSeverity12 = DiagnosticSeverity.WARNING;
            percent3 = DiagnosticsCoreKt.percent(metrics.getRelativeAreaStd());
            assess$add(arrayList, "large_area_uncertainty", diagnosticSeverity12, "Airway-area uncertainty is large", "relative_area_std=" + percent3 + "%", "Do not interpret local constrictions literally; gather a cleaner or calibrated observation.", 0.93f);
        }
        final Comparator comparator = new Comparator() { // from class: org.vocaltract.pixel.OfflineDiagnosticAssistant$assess$$inlined$compareByDescending$1
            /* JADX DEBUG: Multi-variable search result rejected for r1v0, resolved type: T */
            /* JADX DEBUG: Multi-variable search result rejected for r2v0, resolved type: T */
            /* JADX WARN: Multi-variable type inference failed */
            @Override // java.util.Comparator
            public final int compare(T t, T t2) {
                return ComparisonsKt.compareValues(Integer.valueOf(((DiagnosticFinding) t2).getSeverity().ordinal()), Integer.valueOf(((DiagnosticFinding) t).getSeverity().ordinal()));
            }
        };
        List sortedWith = CollectionsKt.sortedWith(arrayList, new Comparator() { // from class: org.vocaltract.pixel.OfflineDiagnosticAssistant$assess$$inlined$thenByDescending$1
            /* JADX DEBUG: Multi-variable search result rejected for r1v0, resolved type: T */
            /* JADX DEBUG: Multi-variable search result rejected for r2v0, resolved type: T */
            /* JADX WARN: Multi-variable type inference failed */
            @Override // java.util.Comparator
            public final int compare(T t, T t2) {
                int compare = comparator.compare(t, t2);
                return compare != 0 ? compare : ComparisonsKt.compareValues(Float.valueOf(((DiagnosticFinding) t2).getConfidence()), Float.valueOf(((DiagnosticFinding) t).getConfidence()));
            }
        });
        List list = sortedWith;
        Iterator it = list.iterator();
        int i2 = 0;
        int i3 = 0;
        while (it.hasNext()) {
            int i4 = WhenMappings.$EnumSwitchMapping$0[((DiagnosticFinding) it.next()).getSeverity().ordinal()];
            if (i4 != 1) {
                i = 2;
                if (i4 == 2) {
                    i = 10;
                } else if (i4 != 3) {
                    throw new NoWhenBranchMatchedException();
                }
            } else {
                i = 25;
            }
            i3 += i;
        }
        int coerceIn = RangesKt.coerceIn(100 - i3, 0, 100);
        boolean z = list instanceof Collection;
        if (!z || !list.isEmpty()) {
            Iterator it2 = list.iterator();
            while (it2.hasNext()) {
                if (((DiagnosticFinding) it2.next()).getSeverity() == DiagnosticSeverity.ERROR) {
                    if (!z || !list.isEmpty()) {
                        Iterator it3 = list.iterator();
                        while (it3.hasNext()) {
                            if (((DiagnosticFinding) it3.next()).getSeverity() == DiagnosticSeverity.ERROR && (i2 = i2 + 1) < 0) {
                                CollectionsKt.throwCountOverflow();
                            }
                        }
                    }
                    str = "Action required: " + i2 + " blocking diagnostic finding(s).";
                    return new DiagnosticAssessment(getProviderId(), nowMillis, coerceIn, str, sortedWith, false, 32, null);
                }
            }
        }
        if (!sortedWith.isEmpty()) {
            str = "Review " + sortedWith.size() + " warning(s) before accepting a refinement.";
        } else {
            str = "No rule-based anomaly is visible in the latest derived metrics.";
        }
        return new DiagnosticAssessment(getProviderId(), nowMillis, coerceIn, str, sortedWith, false, 32, null);
    }

    private static final void assess$add(List<DiagnosticFinding> list, String str, DiagnosticSeverity diagnosticSeverity, String str2, String str3, String str4, float f) {
        list.add(new DiagnosticFinding(str, diagnosticSeverity, str2, str3, str4, RangesKt.coerceIn(f, 0.0f, 1.0f)));
    }
}
