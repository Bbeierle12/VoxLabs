package org.vocaltract.pixel;

import java.util.Arrays;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: VocalAcousticsState.kt */
@Metadata(d1 = {"\u0000,\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\t\n\u0000\n\u0002\u0010\u0017\n\u0000\n\u0002\u0010\u000e\n\u0002\b\r\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B\u001f\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0007¢\u0006\u0004\b\b\u0010\tJ\t\u0010\u0010\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0011\u001a\u00020\u0005HÆ\u0003J\t\u0010\u0012\u001a\u00020\u0007HÆ\u0003J'\u0010\u0013\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u0007HÆ\u0001J\u0013\u0010\u0014\u001a\u00020\u00152\b\u0010\u0016\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0017\u001a\u00020\u0018HÖ\u0001J\t\u0010\u0019\u001a\u00020\u0007HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\n\u0010\u000bR\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\f\u0010\rR\u0011\u0010\u0006\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u000e\u0010\u000f"}, d2 = {"Lorg/vocaltract/pixel/PcmFrame;", "", "timestampNanos", "", "samples", "", "source", "", "<init>", "(J[SLjava/lang/String;)V", "getTimestampNanos", "()J", "getSamples", "()[S", "getSource", "()Ljava/lang/String;", "component1", "component2", "component3", "copy", "equals", "", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class PcmFrame {
    private final short[] samples;
    private final String source;
    private final long timestampNanos;

    public static /* synthetic */ PcmFrame copy$default(PcmFrame pcmFrame, long j, short[] sArr, String str, int i, Object obj) {
        if ((i & 1) != 0) {
            j = pcmFrame.timestampNanos;
        }
        if ((i & 2) != 0) {
            sArr = pcmFrame.samples;
        }
        if ((i & 4) != 0) {
            str = pcmFrame.source;
        }
        return pcmFrame.copy(j, sArr, str);
    }

    /* renamed from: component1, reason: from getter */
    public final long getTimestampNanos() {
        return this.timestampNanos;
    }

    /* renamed from: component2, reason: from getter */
    public final short[] getSamples() {
        return this.samples;
    }

    /* renamed from: component3, reason: from getter */
    public final String getSource() {
        return this.source;
    }

    public final PcmFrame copy(long timestampNanos, short[] samples, String source) {
        Intrinsics.checkNotNullParameter(samples, "samples");
        Intrinsics.checkNotNullParameter(source, "source");
        return new PcmFrame(timestampNanos, samples, source);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof PcmFrame)) {
            return false;
        }
        PcmFrame pcmFrame = (PcmFrame) other;
        return this.timestampNanos == pcmFrame.timestampNanos && Intrinsics.areEqual(this.samples, pcmFrame.samples) && Intrinsics.areEqual(this.source, pcmFrame.source);
    }

    public int hashCode() {
        return (((Long.hashCode(this.timestampNanos) * 31) + Arrays.hashCode(this.samples)) * 31) + this.source.hashCode();
    }

    public String toString() {
        return "PcmFrame(timestampNanos=" + this.timestampNanos + ", samples=" + Arrays.toString(this.samples) + ", source=" + this.source + ")";
    }

    public PcmFrame(long j, short[] samples, String source) {
        Intrinsics.checkNotNullParameter(samples, "samples");
        Intrinsics.checkNotNullParameter(source, "source");
        this.timestampNanos = j;
        this.samples = samples;
        this.source = source;
    }

    public final long getTimestampNanos() {
        return this.timestampNanos;
    }

    public final short[] getSamples() {
        return this.samples;
    }

    public final String getSource() {
        return this.source;
    }
}
