package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.io.ConstantsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.uuid.Uuid;

/* compiled from: VocalAcousticsState.kt */
@Metadata(d1 = {"\u0000B\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\t\n\u0002\b\u0002\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b \n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B]\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0006\u0012\u0006\u0010\u0007\u001a\u00020\b\u0012\u0006\u0010\t\u001a\u00020\u0003\u0012\u0006\u0010\n\u001a\u00020\u000b\u0012\u0006\u0010\f\u001a\u00020\r\u0012\b\b\u0002\u0010\u000e\u001a\u00020\u000f\u0012\b\b\u0002\u0010\u0010\u001a\u00020\b\u0012\b\b\u0002\u0010\u0011\u001a\u00020\b¢\u0006\u0004\b\u0012\u0010\u0013J\t\u0010$\u001a\u00020\u0003HÆ\u0003J\t\u0010%\u001a\u00020\u0003HÆ\u0003J\t\u0010&\u001a\u00020\u0006HÆ\u0003J\t\u0010'\u001a\u00020\bHÆ\u0003J\t\u0010(\u001a\u00020\u0003HÆ\u0003J\t\u0010)\u001a\u00020\u000bHÆ\u0003J\t\u0010*\u001a\u00020\rHÆ\u0003J\t\u0010+\u001a\u00020\u000fHÆ\u0003J\t\u0010,\u001a\u00020\bHÆ\u0003J\t\u0010-\u001a\u00020\bHÆ\u0003Jm\u0010.\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u00062\b\b\u0002\u0010\u0007\u001a\u00020\b2\b\b\u0002\u0010\t\u001a\u00020\u00032\b\b\u0002\u0010\n\u001a\u00020\u000b2\b\b\u0002\u0010\f\u001a\u00020\r2\b\b\u0002\u0010\u000e\u001a\u00020\u000f2\b\b\u0002\u0010\u0010\u001a\u00020\b2\b\b\u0002\u0010\u0011\u001a\u00020\bHÆ\u0001J\u0013\u0010/\u001a\u0002002\b\u00101\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u00102\u001a\u000203HÖ\u0001J\t\u00104\u001a\u00020\u0006HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0015R\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\u0015R\u0011\u0010\u0005\u001a\u00020\u0006¢\u0006\b\n\u0000\u001a\u0004\b\u0017\u0010\u0018R\u0011\u0010\u0007\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u0019\u0010\u001aR\u0011\u0010\t\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u001b\u0010\u0015R\u0011\u0010\n\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\b\u001c\u0010\u001dR\u0011\u0010\f\u001a\u00020\r¢\u0006\b\n\u0000\u001a\u0004\b\u001e\u0010\u001fR\u0011\u0010\u000e\u001a\u00020\u000f¢\u0006\b\n\u0000\u001a\u0004\b \u0010!R\u0011\u0010\u0010\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\"\u0010\u001aR\u0011\u0010\u0011\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b#\u0010\u001a"}, d2 = {"Lorg/vocaltract/pixel/VocalAcousticsState;", "", "sequence", "", "timestampNanos", "source", "", "processingMs", "", "droppedFrames", "acoustic", "Lorg/vocaltract/pixel/AcousticEstimate;", "tract", "Lorg/vocaltract/pixel/TractPosterior;", "articulators", "Lorg/vocaltract/pixel/ArticulatorPosterior;", "analysisWindowMs", "analysisHopMs", "<init>", "(JJLjava/lang/String;FJLorg/vocaltract/pixel/AcousticEstimate;Lorg/vocaltract/pixel/TractPosterior;Lorg/vocaltract/pixel/ArticulatorPosterior;FF)V", "getSequence", "()J", "getTimestampNanos", "getSource", "()Ljava/lang/String;", "getProcessingMs", "()F", "getDroppedFrames", "getAcoustic", "()Lorg/vocaltract/pixel/AcousticEstimate;", "getTract", "()Lorg/vocaltract/pixel/TractPosterior;", "getArticulators", "()Lorg/vocaltract/pixel/ArticulatorPosterior;", "getAnalysisWindowMs", "getAnalysisHopMs", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "component9", "component10", "copy", "equals", "", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class VocalAcousticsState {
    private final AcousticEstimate acoustic;
    private final float analysisHopMs;
    private final float analysisWindowMs;
    private final ArticulatorPosterior articulators;
    private final long droppedFrames;
    private final float processingMs;
    private final long sequence;
    private final String source;
    private final long timestampNanos;
    private final TractPosterior tract;

    /* renamed from: component1, reason: from getter */
    public final long getSequence() {
        return this.sequence;
    }

    /* renamed from: component10, reason: from getter */
    public final float getAnalysisHopMs() {
        return this.analysisHopMs;
    }

    /* renamed from: component2, reason: from getter */
    public final long getTimestampNanos() {
        return this.timestampNanos;
    }

    /* renamed from: component3, reason: from getter */
    public final String getSource() {
        return this.source;
    }

    /* renamed from: component4, reason: from getter */
    public final float getProcessingMs() {
        return this.processingMs;
    }

    /* renamed from: component5, reason: from getter */
    public final long getDroppedFrames() {
        return this.droppedFrames;
    }

    /* renamed from: component6, reason: from getter */
    public final AcousticEstimate getAcoustic() {
        return this.acoustic;
    }

    /* renamed from: component7, reason: from getter */
    public final TractPosterior getTract() {
        return this.tract;
    }

    /* renamed from: component8, reason: from getter */
    public final ArticulatorPosterior getArticulators() {
        return this.articulators;
    }

    /* renamed from: component9, reason: from getter */
    public final float getAnalysisWindowMs() {
        return this.analysisWindowMs;
    }

    public final VocalAcousticsState copy(long sequence, long timestampNanos, String source, float processingMs, long droppedFrames, AcousticEstimate acoustic, TractPosterior tract, ArticulatorPosterior articulators, float analysisWindowMs, float analysisHopMs) {
        Intrinsics.checkNotNullParameter(source, "source");
        Intrinsics.checkNotNullParameter(acoustic, "acoustic");
        Intrinsics.checkNotNullParameter(tract, "tract");
        Intrinsics.checkNotNullParameter(articulators, "articulators");
        return new VocalAcousticsState(sequence, timestampNanos, source, processingMs, droppedFrames, acoustic, tract, articulators, analysisWindowMs, analysisHopMs);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof VocalAcousticsState)) {
            return false;
        }
        VocalAcousticsState vocalAcousticsState = (VocalAcousticsState) other;
        return this.sequence == vocalAcousticsState.sequence && this.timestampNanos == vocalAcousticsState.timestampNanos && Intrinsics.areEqual(this.source, vocalAcousticsState.source) && Float.compare(this.processingMs, vocalAcousticsState.processingMs) == 0 && this.droppedFrames == vocalAcousticsState.droppedFrames && Intrinsics.areEqual(this.acoustic, vocalAcousticsState.acoustic) && Intrinsics.areEqual(this.tract, vocalAcousticsState.tract) && Intrinsics.areEqual(this.articulators, vocalAcousticsState.articulators) && Float.compare(this.analysisWindowMs, vocalAcousticsState.analysisWindowMs) == 0 && Float.compare(this.analysisHopMs, vocalAcousticsState.analysisHopMs) == 0;
    }

    public int hashCode() {
        return (((((((((((((((((Long.hashCode(this.sequence) * 31) + Long.hashCode(this.timestampNanos)) * 31) + this.source.hashCode()) * 31) + Float.hashCode(this.processingMs)) * 31) + Long.hashCode(this.droppedFrames)) * 31) + this.acoustic.hashCode()) * 31) + this.tract.hashCode()) * 31) + this.articulators.hashCode()) * 31) + Float.hashCode(this.analysisWindowMs)) * 31) + Float.hashCode(this.analysisHopMs);
    }

    public String toString() {
        return "VocalAcousticsState(sequence=" + this.sequence + ", timestampNanos=" + this.timestampNanos + ", source=" + this.source + ", processingMs=" + this.processingMs + ", droppedFrames=" + this.droppedFrames + ", acoustic=" + this.acoustic + ", tract=" + this.tract + ", articulators=" + this.articulators + ", analysisWindowMs=" + this.analysisWindowMs + ", analysisHopMs=" + this.analysisHopMs + ")";
    }

    public VocalAcousticsState(long j, long j2, String source, float f, long j3, AcousticEstimate acoustic, TractPosterior tract, ArticulatorPosterior articulators, float f2, float f3) {
        Intrinsics.checkNotNullParameter(source, "source");
        Intrinsics.checkNotNullParameter(acoustic, "acoustic");
        Intrinsics.checkNotNullParameter(tract, "tract");
        Intrinsics.checkNotNullParameter(articulators, "articulators");
        this.sequence = j;
        this.timestampNanos = j2;
        this.source = source;
        this.processingMs = f;
        this.droppedFrames = j3;
        this.acoustic = acoustic;
        this.tract = tract;
        this.articulators = articulators;
        this.analysisWindowMs = f2;
        this.analysisHopMs = f3;
    }

    public final long getSequence() {
        return this.sequence;
    }

    public final long getTimestampNanos() {
        return this.timestampNanos;
    }

    public final String getSource() {
        return this.source;
    }

    public final float getProcessingMs() {
        return this.processingMs;
    }

    public final long getDroppedFrames() {
        return this.droppedFrames;
    }

    public final AcousticEstimate getAcoustic() {
        return this.acoustic;
    }

    public final TractPosterior getTract() {
        return this.tract;
    }

    public /* synthetic */ VocalAcousticsState(long j, long j2, String str, float f, long j3, AcousticEstimate acousticEstimate, TractPosterior tractPosterior, ArticulatorPosterior articulatorPosterior, float f2, float f3, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(j, j2, str, f, j3, acousticEstimate, tractPosterior, (i & Uuid.SIZE_BITS) != 0 ? ArticulatorPosterior.INSTANCE.neutral() : articulatorPosterior, (i & 256) != 0 ? 64.0f : f2, (i & ConstantsKt.MINIMUM_BLOCK_SIZE) != 0 ? 64.0f : f3);
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
