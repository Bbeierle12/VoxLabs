package org.vocaltract.pixel;

import kotlin.Metadata;

/* compiled from: VocalAcousticsState.kt */
@Metadata(d1 = {"\u0000$\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u0007\n\u0002\b\r\n\u0002\u0010\u000b\n\u0002\b\u0003\n\u0002\u0010\u000e\b\u0086\b\u0018\u00002\u00020\u0001B\u001f\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0005¢\u0006\u0004\b\u0007\u0010\bJ\t\u0010\u000e\u001a\u00020\u0003HÆ\u0003J\t\u0010\u000f\u001a\u00020\u0005HÆ\u0003J\t\u0010\u0010\u001a\u00020\u0005HÆ\u0003J'\u0010\u0011\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u0005HÆ\u0001J\u0013\u0010\u0012\u001a\u00020\u00132\b\u0010\u0014\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0015\u001a\u00020\u0003HÖ\u0001J\t\u0010\u0016\u001a\u00020\u0017HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\t\u0010\nR\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\fR\u0011\u0010\u0006\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\r\u0010\f"}, d2 = {"Lorg/vocaltract/pixel/HarmonicEstimate;", "", "number", "", "frequencyHz", "", "relativeGainDb", "<init>", "(IFF)V", "getNumber", "()I", "getFrequencyHz", "()F", "getRelativeGainDb", "component1", "component2", "component3", "copy", "equals", "", "other", "hashCode", "toString", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class HarmonicEstimate {
    private final float frequencyHz;
    private final int number;
    private final float relativeGainDb;

    public static /* synthetic */ HarmonicEstimate copy$default(HarmonicEstimate harmonicEstimate, int i, float f, float f2, int i2, Object obj) {
        if ((i2 & 1) != 0) {
            i = harmonicEstimate.number;
        }
        if ((i2 & 2) != 0) {
            f = harmonicEstimate.frequencyHz;
        }
        if ((i2 & 4) != 0) {
            f2 = harmonicEstimate.relativeGainDb;
        }
        return harmonicEstimate.copy(i, f, f2);
    }

    /* renamed from: component1, reason: from getter */
    public final int getNumber() {
        return this.number;
    }

    /* renamed from: component2, reason: from getter */
    public final float getFrequencyHz() {
        return this.frequencyHz;
    }

    /* renamed from: component3, reason: from getter */
    public final float getRelativeGainDb() {
        return this.relativeGainDb;
    }

    public final HarmonicEstimate copy(int number, float frequencyHz, float relativeGainDb) {
        return new HarmonicEstimate(number, frequencyHz, relativeGainDb);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof HarmonicEstimate)) {
            return false;
        }
        HarmonicEstimate harmonicEstimate = (HarmonicEstimate) other;
        return this.number == harmonicEstimate.number && Float.compare(this.frequencyHz, harmonicEstimate.frequencyHz) == 0 && Float.compare(this.relativeGainDb, harmonicEstimate.relativeGainDb) == 0;
    }

    public int hashCode() {
        return (((Integer.hashCode(this.number) * 31) + Float.hashCode(this.frequencyHz)) * 31) + Float.hashCode(this.relativeGainDb);
    }

    public String toString() {
        return "HarmonicEstimate(number=" + this.number + ", frequencyHz=" + this.frequencyHz + ", relativeGainDb=" + this.relativeGainDb + ")";
    }

    public HarmonicEstimate(int i, float f, float f2) {
        this.number = i;
        this.frequencyHz = f;
        this.relativeGainDb = f2;
    }

    public final int getNumber() {
        return this.number;
    }

    public final float getFrequencyHz() {
        return this.frequencyHz;
    }

    public final float getRelativeGainDb() {
        return this.relativeGainDb;
    }
}
