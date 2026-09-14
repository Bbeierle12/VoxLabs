package org.vocaltract.pixel;

import java.util.List;
import kotlin.Metadata;
import kotlin.collections.CollectionsKt;
import kotlin.io.ConstantsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.uuid.Uuid;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u0000<\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\t\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0003\n\u0002\u0010\u000b\n\u0002\b\u0012\n\u0002\u0010 \n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0002\bM\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001BÃ\u0002\u0012\b\b\u0002\u0010\u0002\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0004\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0006\u001a\u00020\u0007\u0012\b\b\u0002\u0010\b\u001a\u00020\u0007\u0012\b\b\u0002\u0010\t\u001a\u00020\u0003\u0012\b\b\u0002\u0010\n\u001a\u00020\u000b\u0012\n\b\u0002\u0010\f\u001a\u0004\u0018\u00010\u0007\u0012\b\b\u0002\u0010\r\u001a\u00020\u0007\u0012\b\b\u0002\u0010\u000e\u001a\u00020\u0007\u0012\b\b\u0002\u0010\u000f\u001a\u00020\u0007\u0012\b\b\u0002\u0010\u0010\u001a\u00020\u0007\u0012\b\b\u0002\u0010\u0011\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0012\u001a\u00020\u000b\u0012\b\b\u0002\u0010\u0013\u001a\u00020\u000b\u0012\n\b\u0002\u0010\u0014\u001a\u0004\u0018\u00010\u0007\u0012\b\b\u0002\u0010\u0015\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0016\u001a\u00020\u000b\u0012\b\b\u0002\u0010\u0017\u001a\u00020\u0007\u0012\b\b\u0002\u0010\u0018\u001a\u00020\u0007\u0012\b\b\u0002\u0010\u0019\u001a\u00020\u0007\u0012\b\b\u0002\u0010\u001a\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u001b\u001a\u00020\u0007\u0012\b\b\u0002\u0010\u001c\u001a\u00020\u000b\u0012\u000e\b\u0002\u0010\u001d\u001a\b\u0012\u0004\u0012\u00020\u00070\u001e\u0012\b\b\u0002\u0010\u001f\u001a\u00020\u000b\u0012\b\b\u0002\u0010 \u001a\u00020\u0005\u0012\u000e\b\u0002\u0010!\u001a\b\u0012\u0004\u0012\u00020\u00070\u001e\u0012\b\b\u0002\u0010\"\u001a\u00020#\u0012\b\b\u0002\u0010$\u001a\u00020\u0007\u0012\b\b\u0002\u0010%\u001a\u00020\u0007¢\u0006\u0004\b&\u0010'J\t\u0010N\u001a\u00020\u0003HÆ\u0003J\t\u0010O\u001a\u00020\u0005HÆ\u0003J\t\u0010P\u001a\u00020\u0007HÆ\u0003J\t\u0010Q\u001a\u00020\u0007HÆ\u0003J\t\u0010R\u001a\u00020\u0003HÆ\u0003J\t\u0010S\u001a\u00020\u000bHÆ\u0003J\u0010\u0010T\u001a\u0004\u0018\u00010\u0007HÆ\u0003¢\u0006\u0002\u00103J\t\u0010U\u001a\u00020\u0007HÆ\u0003J\t\u0010V\u001a\u00020\u0007HÆ\u0003J\t\u0010W\u001a\u00020\u0007HÆ\u0003J\t\u0010X\u001a\u00020\u0007HÆ\u0003J\t\u0010Y\u001a\u00020\u0005HÆ\u0003J\t\u0010Z\u001a\u00020\u000bHÆ\u0003J\t\u0010[\u001a\u00020\u000bHÆ\u0003J\u0010\u0010\\\u001a\u0004\u0018\u00010\u0007HÆ\u0003¢\u0006\u0002\u00103J\t\u0010]\u001a\u00020\u0005HÆ\u0003J\t\u0010^\u001a\u00020\u000bHÆ\u0003J\t\u0010_\u001a\u00020\u0007HÆ\u0003J\t\u0010`\u001a\u00020\u0007HÆ\u0003J\t\u0010a\u001a\u00020\u0007HÆ\u0003J\t\u0010b\u001a\u00020\u0005HÆ\u0003J\t\u0010c\u001a\u00020\u0007HÆ\u0003J\t\u0010d\u001a\u00020\u000bHÆ\u0003J\u000f\u0010e\u001a\b\u0012\u0004\u0012\u00020\u00070\u001eHÆ\u0003J\t\u0010f\u001a\u00020\u000bHÆ\u0003J\t\u0010g\u001a\u00020\u0005HÆ\u0003J\u000f\u0010h\u001a\b\u0012\u0004\u0012\u00020\u00070\u001eHÆ\u0003J\t\u0010i\u001a\u00020#HÆ\u0003J\t\u0010j\u001a\u00020\u0007HÆ\u0003J\t\u0010k\u001a\u00020\u0007HÆ\u0003JÊ\u0002\u0010l\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00072\b\b\u0002\u0010\b\u001a\u00020\u00072\b\b\u0002\u0010\t\u001a\u00020\u00032\b\b\u0002\u0010\n\u001a\u00020\u000b2\n\b\u0002\u0010\f\u001a\u0004\u0018\u00010\u00072\b\b\u0002\u0010\r\u001a\u00020\u00072\b\b\u0002\u0010\u000e\u001a\u00020\u00072\b\b\u0002\u0010\u000f\u001a\u00020\u00072\b\b\u0002\u0010\u0010\u001a\u00020\u00072\b\b\u0002\u0010\u0011\u001a\u00020\u00052\b\b\u0002\u0010\u0012\u001a\u00020\u000b2\b\b\u0002\u0010\u0013\u001a\u00020\u000b2\n\b\u0002\u0010\u0014\u001a\u0004\u0018\u00010\u00072\b\b\u0002\u0010\u0015\u001a\u00020\u00052\b\b\u0002\u0010\u0016\u001a\u00020\u000b2\b\b\u0002\u0010\u0017\u001a\u00020\u00072\b\b\u0002\u0010\u0018\u001a\u00020\u00072\b\b\u0002\u0010\u0019\u001a\u00020\u00072\b\b\u0002\u0010\u001a\u001a\u00020\u00052\b\b\u0002\u0010\u001b\u001a\u00020\u00072\b\b\u0002\u0010\u001c\u001a\u00020\u000b2\u000e\b\u0002\u0010\u001d\u001a\b\u0012\u0004\u0012\u00020\u00070\u001e2\b\b\u0002\u0010\u001f\u001a\u00020\u000b2\b\b\u0002\u0010 \u001a\u00020\u00052\u000e\b\u0002\u0010!\u001a\b\u0012\u0004\u0012\u00020\u00070\u001e2\b\b\u0002\u0010\"\u001a\u00020#2\b\b\u0002\u0010$\u001a\u00020\u00072\b\b\u0002\u0010%\u001a\u00020\u0007HÆ\u0001¢\u0006\u0002\u0010mJ\u0013\u0010n\u001a\u00020\u000b2\b\u0010o\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010p\u001a\u00020qHÖ\u0001J\t\u0010r\u001a\u00020\u0005HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b(\u0010)R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b*\u0010+R\u0011\u0010\u0006\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b,\u0010-R\u0011\u0010\b\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b.\u0010-R\u0011\u0010\t\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b/\u0010)R\u0011\u0010\n\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\b0\u00101R\u0015\u0010\f\u001a\u0004\u0018\u00010\u0007¢\u0006\n\n\u0002\u00104\u001a\u0004\b2\u00103R\u0011\u0010\r\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b5\u0010-R\u0011\u0010\u000e\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b6\u0010-R\u0011\u0010\u000f\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b7\u0010-R\u0011\u0010\u0010\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b8\u0010-R\u0011\u0010\u0011\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b9\u0010+R\u0011\u0010\u0012\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\b:\u00101R\u0011\u0010\u0013\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\b;\u00101R\u0015\u0010\u0014\u001a\u0004\u0018\u00010\u0007¢\u0006\n\n\u0002\u00104\u001a\u0004\b<\u00103R\u0011\u0010\u0015\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b=\u0010+R\u0011\u0010\u0016\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\b>\u00101R\u0011\u0010\u0017\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b?\u0010-R\u0011\u0010\u0018\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b@\u0010-R\u0011\u0010\u0019\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\bA\u0010-R\u0011\u0010\u001a\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\bB\u0010+R\u0011\u0010\u001b\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\bC\u0010-R\u0011\u0010\u001c\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\bD\u00101R\u0017\u0010\u001d\u001a\b\u0012\u0004\u0012\u00020\u00070\u001e¢\u0006\b\n\u0000\u001a\u0004\bE\u0010FR\u0011\u0010\u001f\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\bG\u00101R\u0011\u0010 \u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\bH\u0010+R\u0017\u0010!\u001a\b\u0012\u0004\u0012\u00020\u00070\u001e¢\u0006\b\n\u0000\u001a\u0004\bI\u0010FR\u0011\u0010\"\u001a\u00020#¢\u0006\b\n\u0000\u001a\u0004\bJ\u0010KR\u0011\u0010$\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\bL\u0010-R\u0011\u0010%\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\bM\u0010-"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticMetrics;", "", "sequence", "", "source", "", "processingMs", "", "frameBudgetMs", "droppedFrames", "voiced", "", "f0Hz", "f0Confidence", "meanFormantStdHz", "tractConfidence", "relativeAreaStd", "rendererMode", "microphoneGranted", "synthesizerActive", "rawF0Hz", "pitchDecision", "pitchRejected", "harmonicity", "snrDb", "noiseFloorDb", "noiseState", "noiseConfidence", "backgroundChanged", "noiseBandsDb", "", "posteriorAbstained", "abstentionReason", "formantCandidatesHz", "articulators", "Lorg/vocaltract/pixel/ArticulatorPosterior;", "analysisWindowMs", "analysisHopMs", "<init>", "(JLjava/lang/String;FFJZLjava/lang/Float;FFFFLjava/lang/String;ZZLjava/lang/Float;Ljava/lang/String;ZFFFLjava/lang/String;FZLjava/util/List;ZLjava/lang/String;Ljava/util/List;Lorg/vocaltract/pixel/ArticulatorPosterior;FF)V", "getSequence", "()J", "getSource", "()Ljava/lang/String;", "getProcessingMs", "()F", "getFrameBudgetMs", "getDroppedFrames", "getVoiced", "()Z", "getF0Hz", "()Ljava/lang/Float;", "Ljava/lang/Float;", "getF0Confidence", "getMeanFormantStdHz", "getTractConfidence", "getRelativeAreaStd", "getRendererMode", "getMicrophoneGranted", "getSynthesizerActive", "getRawF0Hz", "getPitchDecision", "getPitchRejected", "getHarmonicity", "getSnrDb", "getNoiseFloorDb", "getNoiseState", "getNoiseConfidence", "getBackgroundChanged", "getNoiseBandsDb", "()Ljava/util/List;", "getPosteriorAbstained", "getAbstentionReason", "getFormantCandidatesHz", "getArticulators", "()Lorg/vocaltract/pixel/ArticulatorPosterior;", "getAnalysisWindowMs", "getAnalysisHopMs", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "component9", "component10", "component11", "component12", "component13", "component14", "component15", "component16", "component17", "component18", "component19", "component20", "component21", "component22", "component23", "component24", "component25", "component26", "component27", "component28", "component29", "component30", "copy", "(JLjava/lang/String;FFJZLjava/lang/Float;FFFFLjava/lang/String;ZZLjava/lang/Float;Ljava/lang/String;ZFFFLjava/lang/String;FZLjava/util/List;ZLjava/lang/String;Ljava/util/List;Lorg/vocaltract/pixel/ArticulatorPosterior;FF)Lorg/vocaltract/pixel/DiagnosticMetrics;", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class DiagnosticMetrics {
    private final String abstentionReason;
    private final float analysisHopMs;
    private final float analysisWindowMs;
    private final ArticulatorPosterior articulators;
    private final boolean backgroundChanged;
    private final long droppedFrames;
    private final float f0Confidence;
    private final Float f0Hz;
    private final List<Float> formantCandidatesHz;
    private final float frameBudgetMs;
    private final float harmonicity;
    private final float meanFormantStdHz;
    private final boolean microphoneGranted;
    private final List<Float> noiseBandsDb;
    private final float noiseConfidence;
    private final float noiseFloorDb;
    private final String noiseState;
    private final String pitchDecision;
    private final boolean pitchRejected;
    private final boolean posteriorAbstained;
    private final float processingMs;
    private final Float rawF0Hz;
    private final float relativeAreaStd;
    private final String rendererMode;
    private final long sequence;
    private final float snrDb;
    private final String source;
    private final boolean synthesizerActive;
    private final float tractConfidence;
    private final boolean voiced;

    public DiagnosticMetrics() {
        this(0L, null, 0.0f, 0.0f, 0L, false, null, 0.0f, 0.0f, 0.0f, 0.0f, null, false, false, null, null, false, 0.0f, 0.0f, 0.0f, null, 0.0f, false, null, false, null, null, null, 0.0f, 0.0f, 1073741823, null);
    }

    /* renamed from: component1, reason: from getter */
    public final long getSequence() {
        return this.sequence;
    }

    /* renamed from: component10, reason: from getter */
    public final float getTractConfidence() {
        return this.tractConfidence;
    }

    /* renamed from: component11, reason: from getter */
    public final float getRelativeAreaStd() {
        return this.relativeAreaStd;
    }

    /* renamed from: component12, reason: from getter */
    public final String getRendererMode() {
        return this.rendererMode;
    }

    /* renamed from: component13, reason: from getter */
    public final boolean getMicrophoneGranted() {
        return this.microphoneGranted;
    }

    /* renamed from: component14, reason: from getter */
    public final boolean getSynthesizerActive() {
        return this.synthesizerActive;
    }

    /* renamed from: component15, reason: from getter */
    public final Float getRawF0Hz() {
        return this.rawF0Hz;
    }

    /* renamed from: component16, reason: from getter */
    public final String getPitchDecision() {
        return this.pitchDecision;
    }

    /* renamed from: component17, reason: from getter */
    public final boolean getPitchRejected() {
        return this.pitchRejected;
    }

    /* renamed from: component18, reason: from getter */
    public final float getHarmonicity() {
        return this.harmonicity;
    }

    /* renamed from: component19, reason: from getter */
    public final float getSnrDb() {
        return this.snrDb;
    }

    /* renamed from: component2, reason: from getter */
    public final String getSource() {
        return this.source;
    }

    /* renamed from: component20, reason: from getter */
    public final float getNoiseFloorDb() {
        return this.noiseFloorDb;
    }

    /* renamed from: component21, reason: from getter */
    public final String getNoiseState() {
        return this.noiseState;
    }

    /* renamed from: component22, reason: from getter */
    public final float getNoiseConfidence() {
        return this.noiseConfidence;
    }

    /* renamed from: component23, reason: from getter */
    public final boolean getBackgroundChanged() {
        return this.backgroundChanged;
    }

    public final List<Float> component24() {
        return this.noiseBandsDb;
    }

    /* renamed from: component25, reason: from getter */
    public final boolean getPosteriorAbstained() {
        return this.posteriorAbstained;
    }

    /* renamed from: component26, reason: from getter */
    public final String getAbstentionReason() {
        return this.abstentionReason;
    }

    public final List<Float> component27() {
        return this.formantCandidatesHz;
    }

    /* renamed from: component28, reason: from getter */
    public final ArticulatorPosterior getArticulators() {
        return this.articulators;
    }

    /* renamed from: component29, reason: from getter */
    public final float getAnalysisWindowMs() {
        return this.analysisWindowMs;
    }

    /* renamed from: component3, reason: from getter */
    public final float getProcessingMs() {
        return this.processingMs;
    }

    /* renamed from: component30, reason: from getter */
    public final float getAnalysisHopMs() {
        return this.analysisHopMs;
    }

    /* renamed from: component4, reason: from getter */
    public final float getFrameBudgetMs() {
        return this.frameBudgetMs;
    }

    /* renamed from: component5, reason: from getter */
    public final long getDroppedFrames() {
        return this.droppedFrames;
    }

    /* renamed from: component6, reason: from getter */
    public final boolean getVoiced() {
        return this.voiced;
    }

    /* renamed from: component7, reason: from getter */
    public final Float getF0Hz() {
        return this.f0Hz;
    }

    /* renamed from: component8, reason: from getter */
    public final float getF0Confidence() {
        return this.f0Confidence;
    }

    /* renamed from: component9, reason: from getter */
    public final float getMeanFormantStdHz() {
        return this.meanFormantStdHz;
    }

    public final DiagnosticMetrics copy(long sequence, String source, float processingMs, float frameBudgetMs, long droppedFrames, boolean voiced, Float f0Hz, float f0Confidence, float meanFormantStdHz, float tractConfidence, float relativeAreaStd, String rendererMode, boolean microphoneGranted, boolean synthesizerActive, Float rawF0Hz, String pitchDecision, boolean pitchRejected, float harmonicity, float snrDb, float noiseFloorDb, String noiseState, float noiseConfidence, boolean backgroundChanged, List<Float> noiseBandsDb, boolean posteriorAbstained, String abstentionReason, List<Float> formantCandidatesHz, ArticulatorPosterior articulators, float analysisWindowMs, float analysisHopMs) {
        Intrinsics.checkNotNullParameter(source, "source");
        Intrinsics.checkNotNullParameter(rendererMode, "rendererMode");
        Intrinsics.checkNotNullParameter(pitchDecision, "pitchDecision");
        Intrinsics.checkNotNullParameter(noiseState, "noiseState");
        Intrinsics.checkNotNullParameter(noiseBandsDb, "noiseBandsDb");
        Intrinsics.checkNotNullParameter(abstentionReason, "abstentionReason");
        Intrinsics.checkNotNullParameter(formantCandidatesHz, "formantCandidatesHz");
        Intrinsics.checkNotNullParameter(articulators, "articulators");
        return new DiagnosticMetrics(sequence, source, processingMs, frameBudgetMs, droppedFrames, voiced, f0Hz, f0Confidence, meanFormantStdHz, tractConfidence, relativeAreaStd, rendererMode, microphoneGranted, synthesizerActive, rawF0Hz, pitchDecision, pitchRejected, harmonicity, snrDb, noiseFloorDb, noiseState, noiseConfidence, backgroundChanged, noiseBandsDb, posteriorAbstained, abstentionReason, formantCandidatesHz, articulators, analysisWindowMs, analysisHopMs);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof DiagnosticMetrics)) {
            return false;
        }
        DiagnosticMetrics diagnosticMetrics = (DiagnosticMetrics) other;
        return this.sequence == diagnosticMetrics.sequence && Intrinsics.areEqual(this.source, diagnosticMetrics.source) && Float.compare(this.processingMs, diagnosticMetrics.processingMs) == 0 && Float.compare(this.frameBudgetMs, diagnosticMetrics.frameBudgetMs) == 0 && this.droppedFrames == diagnosticMetrics.droppedFrames && this.voiced == diagnosticMetrics.voiced && Intrinsics.areEqual((Object) this.f0Hz, (Object) diagnosticMetrics.f0Hz) && Float.compare(this.f0Confidence, diagnosticMetrics.f0Confidence) == 0 && Float.compare(this.meanFormantStdHz, diagnosticMetrics.meanFormantStdHz) == 0 && Float.compare(this.tractConfidence, diagnosticMetrics.tractConfidence) == 0 && Float.compare(this.relativeAreaStd, diagnosticMetrics.relativeAreaStd) == 0 && Intrinsics.areEqual(this.rendererMode, diagnosticMetrics.rendererMode) && this.microphoneGranted == diagnosticMetrics.microphoneGranted && this.synthesizerActive == diagnosticMetrics.synthesizerActive && Intrinsics.areEqual((Object) this.rawF0Hz, (Object) diagnosticMetrics.rawF0Hz) && Intrinsics.areEqual(this.pitchDecision, diagnosticMetrics.pitchDecision) && this.pitchRejected == diagnosticMetrics.pitchRejected && Float.compare(this.harmonicity, diagnosticMetrics.harmonicity) == 0 && Float.compare(this.snrDb, diagnosticMetrics.snrDb) == 0 && Float.compare(this.noiseFloorDb, diagnosticMetrics.noiseFloorDb) == 0 && Intrinsics.areEqual(this.noiseState, diagnosticMetrics.noiseState) && Float.compare(this.noiseConfidence, diagnosticMetrics.noiseConfidence) == 0 && this.backgroundChanged == diagnosticMetrics.backgroundChanged && Intrinsics.areEqual(this.noiseBandsDb, diagnosticMetrics.noiseBandsDb) && this.posteriorAbstained == diagnosticMetrics.posteriorAbstained && Intrinsics.areEqual(this.abstentionReason, diagnosticMetrics.abstentionReason) && Intrinsics.areEqual(this.formantCandidatesHz, diagnosticMetrics.formantCandidatesHz) && Intrinsics.areEqual(this.articulators, diagnosticMetrics.articulators) && Float.compare(this.analysisWindowMs, diagnosticMetrics.analysisWindowMs) == 0 && Float.compare(this.analysisHopMs, diagnosticMetrics.analysisHopMs) == 0;
    }

    public int hashCode() {
        int hashCode = ((((((((((Long.hashCode(this.sequence) * 31) + this.source.hashCode()) * 31) + Float.hashCode(this.processingMs)) * 31) + Float.hashCode(this.frameBudgetMs)) * 31) + Long.hashCode(this.droppedFrames)) * 31) + Boolean.hashCode(this.voiced)) * 31;
        Float f = this.f0Hz;
        int hashCode2 = (((((((((((((((hashCode + (f == null ? 0 : f.hashCode())) * 31) + Float.hashCode(this.f0Confidence)) * 31) + Float.hashCode(this.meanFormantStdHz)) * 31) + Float.hashCode(this.tractConfidence)) * 31) + Float.hashCode(this.relativeAreaStd)) * 31) + this.rendererMode.hashCode()) * 31) + Boolean.hashCode(this.microphoneGranted)) * 31) + Boolean.hashCode(this.synthesizerActive)) * 31;
        Float f2 = this.rawF0Hz;
        return ((((((((((((((((((((((((((((((hashCode2 + (f2 != null ? f2.hashCode() : 0)) * 31) + this.pitchDecision.hashCode()) * 31) + Boolean.hashCode(this.pitchRejected)) * 31) + Float.hashCode(this.harmonicity)) * 31) + Float.hashCode(this.snrDb)) * 31) + Float.hashCode(this.noiseFloorDb)) * 31) + this.noiseState.hashCode()) * 31) + Float.hashCode(this.noiseConfidence)) * 31) + Boolean.hashCode(this.backgroundChanged)) * 31) + this.noiseBandsDb.hashCode()) * 31) + Boolean.hashCode(this.posteriorAbstained)) * 31) + this.abstentionReason.hashCode()) * 31) + this.formantCandidatesHz.hashCode()) * 31) + this.articulators.hashCode()) * 31) + Float.hashCode(this.analysisWindowMs)) * 31) + Float.hashCode(this.analysisHopMs);
    }

    public String toString() {
        return "DiagnosticMetrics(sequence=" + this.sequence + ", source=" + this.source + ", processingMs=" + this.processingMs + ", frameBudgetMs=" + this.frameBudgetMs + ", droppedFrames=" + this.droppedFrames + ", voiced=" + this.voiced + ", f0Hz=" + this.f0Hz + ", f0Confidence=" + this.f0Confidence + ", meanFormantStdHz=" + this.meanFormantStdHz + ", tractConfidence=" + this.tractConfidence + ", relativeAreaStd=" + this.relativeAreaStd + ", rendererMode=" + this.rendererMode + ", microphoneGranted=" + this.microphoneGranted + ", synthesizerActive=" + this.synthesizerActive + ", rawF0Hz=" + this.rawF0Hz + ", pitchDecision=" + this.pitchDecision + ", pitchRejected=" + this.pitchRejected + ", harmonicity=" + this.harmonicity + ", snrDb=" + this.snrDb + ", noiseFloorDb=" + this.noiseFloorDb + ", noiseState=" + this.noiseState + ", noiseConfidence=" + this.noiseConfidence + ", backgroundChanged=" + this.backgroundChanged + ", noiseBandsDb=" + this.noiseBandsDb + ", posteriorAbstained=" + this.posteriorAbstained + ", abstentionReason=" + this.abstentionReason + ", formantCandidatesHz=" + this.formantCandidatesHz + ", articulators=" + this.articulators + ", analysisWindowMs=" + this.analysisWindowMs + ", analysisHopMs=" + this.analysisHopMs + ")";
    }

    public DiagnosticMetrics(long j, String source, float f, float f2, long j2, boolean z, Float f3, float f4, float f5, float f6, float f7, String rendererMode, boolean z2, boolean z3, Float f8, String pitchDecision, boolean z4, float f9, float f10, float f11, String noiseState, float f12, boolean z5, List<Float> noiseBandsDb, boolean z6, String abstentionReason, List<Float> formantCandidatesHz, ArticulatorPosterior articulators, float f13, float f14) {
        Intrinsics.checkNotNullParameter(source, "source");
        Intrinsics.checkNotNullParameter(rendererMode, "rendererMode");
        Intrinsics.checkNotNullParameter(pitchDecision, "pitchDecision");
        Intrinsics.checkNotNullParameter(noiseState, "noiseState");
        Intrinsics.checkNotNullParameter(noiseBandsDb, "noiseBandsDb");
        Intrinsics.checkNotNullParameter(abstentionReason, "abstentionReason");
        Intrinsics.checkNotNullParameter(formantCandidatesHz, "formantCandidatesHz");
        Intrinsics.checkNotNullParameter(articulators, "articulators");
        this.sequence = j;
        this.source = source;
        this.processingMs = f;
        this.frameBudgetMs = f2;
        this.droppedFrames = j2;
        this.voiced = z;
        this.f0Hz = f3;
        this.f0Confidence = f4;
        this.meanFormantStdHz = f5;
        this.tractConfidence = f6;
        this.relativeAreaStd = f7;
        this.rendererMode = rendererMode;
        this.microphoneGranted = z2;
        this.synthesizerActive = z3;
        this.rawF0Hz = f8;
        this.pitchDecision = pitchDecision;
        this.pitchRejected = z4;
        this.harmonicity = f9;
        this.snrDb = f10;
        this.noiseFloorDb = f11;
        this.noiseState = noiseState;
        this.noiseConfidence = f12;
        this.backgroundChanged = z5;
        this.noiseBandsDb = noiseBandsDb;
        this.posteriorAbstained = z6;
        this.abstentionReason = abstentionReason;
        this.formantCandidatesHz = formantCandidatesHz;
        this.articulators = articulators;
        this.analysisWindowMs = f13;
        this.analysisHopMs = f14;
    }

    public final long getSequence() {
        return this.sequence;
    }

    /* JADX WARN: Illegal instructions before constructor call */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public /* synthetic */ DiagnosticMetrics(long j, String str, float f, float f2, long j2, boolean z, Float f3, float f4, float f5, float f6, float f7, String str2, boolean z2, boolean z3, Float f8, String str3, boolean z4, float f9, float f10, float f11, String str4, float f12, boolean z5, List list, boolean z6, String str5, List list2, ArticulatorPosterior articulatorPosterior, float f13, float f14, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(r4, r1, r6, r8, r2, r10, r12, (i & Uuid.SIZE_BITS) != 0 ? 0.0f : f4, (i & 256) != 0 ? 0.0f : f5, (i & ConstantsKt.MINIMUM_BLOCK_SIZE) != 0 ? 0.0f : f6, (i & 1024) != 0 ? 1.0f : f7, (i & 2048) != 0 ? "initializing" : str2, (i & ConstantsKt.DEFAULT_BLOCK_SIZE) != 0 ? false : z2, (i & ConstantsKt.DEFAULT_BUFFER_SIZE) != 0 ? false : z3, (i & 16384) != 0 ? r12 : f8, (i & 32768) != 0 ? "unprocessed" : str3, (i & 65536) != 0 ? false : z4, (i & 131072) != 0 ? 0.0f : f9, (i & 262144) != 0 ? 30.0f : f10, (i & 524288) != 0 ? -120.0f : f11, (i & 1048576) != 0 ? "stable" : str4, (i & 2097152) == 0 ? f12 : 1.0f, (i & 4194304) != 0 ? false : z5, (i & 8388608) != 0 ? CollectionsKt.emptyList() : list, (i & 16777216) != 0 ? false : z6, (i & 33554432) != 0 ? "none" : str5, (i & 67108864) != 0 ? CollectionsKt.emptyList() : list2, (i & 134217728) != 0 ? ArticulatorPosterior.INSTANCE.neutral() : articulatorPosterior, (i & 268435456) != 0 ? 64.0f : f13, (i & 536870912) != 0 ? 64.0f : f14);
        long j3 = (i & 1) != 0 ? 0L : j;
        String str6 = (i & 2) != 0 ? "idle" : str;
        float f15 = (i & 4) != 0 ? 0.0f : f;
        float f16 = (i & 8) != 0 ? 64.0f : f2;
        long j4 = (i & 16) == 0 ? j2 : 0L;
        boolean z7 = (i & 32) != 0 ? false : z;
        Float f17 = (i & 64) != 0 ? null : f3;
    }

    public final String getSource() {
        return this.source;
    }

    public final float getProcessingMs() {
        return this.processingMs;
    }

    public final float getFrameBudgetMs() {
        return this.frameBudgetMs;
    }

    public final long getDroppedFrames() {
        return this.droppedFrames;
    }

    public final boolean getVoiced() {
        return this.voiced;
    }

    public final Float getF0Hz() {
        return this.f0Hz;
    }

    public final float getF0Confidence() {
        return this.f0Confidence;
    }

    public final float getMeanFormantStdHz() {
        return this.meanFormantStdHz;
    }

    public final float getTractConfidence() {
        return this.tractConfidence;
    }

    public final float getRelativeAreaStd() {
        return this.relativeAreaStd;
    }

    public final String getRendererMode() {
        return this.rendererMode;
    }

    public final boolean getMicrophoneGranted() {
        return this.microphoneGranted;
    }

    public final boolean getSynthesizerActive() {
        return this.synthesizerActive;
    }

    public final Float getRawF0Hz() {
        return this.rawF0Hz;
    }

    public final String getPitchDecision() {
        return this.pitchDecision;
    }

    public final boolean getPitchRejected() {
        return this.pitchRejected;
    }

    public final float getHarmonicity() {
        return this.harmonicity;
    }

    public final float getSnrDb() {
        return this.snrDb;
    }

    public final float getNoiseFloorDb() {
        return this.noiseFloorDb;
    }

    public final String getNoiseState() {
        return this.noiseState;
    }

    public final float getNoiseConfidence() {
        return this.noiseConfidence;
    }

    public final boolean getBackgroundChanged() {
        return this.backgroundChanged;
    }

    public final List<Float> getNoiseBandsDb() {
        return this.noiseBandsDb;
    }

    public final boolean getPosteriorAbstained() {
        return this.posteriorAbstained;
    }

    public final String getAbstentionReason() {
        return this.abstentionReason;
    }

    public final List<Float> getFormantCandidatesHz() {
        return this.formantCandidatesHz;
    }

    public final ArticulatorPosterior getArticulators() {
        return this.articulators;
    }

    public final float getAnalysisWindowMs() {
        return this.analysisWindowMs;
    }

    public final float getAnalysisHopMs() {
        return this.analysisHopMs;
    }
}
