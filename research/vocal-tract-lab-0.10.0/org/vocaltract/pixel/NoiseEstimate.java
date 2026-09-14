package org.vocaltract.pixel;

import java.util.Arrays;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: InferenceStabilizers.kt */
@Metadata(d1 = {"\u0000.\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0014\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0002\n\u0002\u0010\u000e\n\u0002\b\u0002\n\u0002\u0010\u000b\n\u0002\b\u0019\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B?\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0005\u0012\u0006\u0010\u0007\u001a\u00020\b\u0012\u0006\u0010\t\u001a\u00020\u0005\u0012\u0006\u0010\n\u001a\u00020\u000b\u0012\u0006\u0010\f\u001a\u00020\u0003¢\u0006\u0004\b\r\u0010\u000eJ\t\u0010\u001a\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001b\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001c\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001d\u001a\u00020\bHÆ\u0003J\t\u0010\u001e\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001f\u001a\u00020\u000bHÆ\u0003J\t\u0010 \u001a\u00020\u0003HÆ\u0003JO\u0010!\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00052\b\b\u0002\u0010\u0007\u001a\u00020\b2\b\b\u0002\u0010\t\u001a\u00020\u00052\b\b\u0002\u0010\n\u001a\u00020\u000b2\b\b\u0002\u0010\f\u001a\u00020\u0003HÆ\u0001J\u0013\u0010\"\u001a\u00020\u000b2\b\u0010#\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010$\u001a\u00020%HÖ\u0001J\t\u0010&\u001a\u00020\bHÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u0010R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0011\u0010\u0012R\u0011\u0010\u0006\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0013\u0010\u0012R\u0011\u0010\u0007\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0015R\u0011\u0010\t\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\u0012R\u0011\u0010\n\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\b\u0017\u0010\u0018R\u0011\u0010\f\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0019\u0010\u0010"}, d2 = {"Lorg/vocaltract/pixel/NoiseEstimate;", "", "cleanPower", "", "snrDb", "", "noiseFloorDb", "state", "", "confidence", "changed", "", "bandsDb", "<init>", "([FFFLjava/lang/String;FZ[F)V", "getCleanPower", "()[F", "getSnrDb", "()F", "getNoiseFloorDb", "getState", "()Ljava/lang/String;", "getConfidence", "getChanged", "()Z", "getBandsDb", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "copy", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class NoiseEstimate {
    private final float[] bandsDb;
    private final boolean changed;
    private final float[] cleanPower;
    private final float confidence;
    private final float noiseFloorDb;
    private final float snrDb;
    private final String state;

    public static /* synthetic */ NoiseEstimate copy$default(NoiseEstimate noiseEstimate, float[] fArr, float f, float f2, String str, float f3, boolean z, float[] fArr2, int i, Object obj) {
        if ((i & 1) != 0) {
            fArr = noiseEstimate.cleanPower;
        }
        if ((i & 2) != 0) {
            f = noiseEstimate.snrDb;
        }
        float f4 = f;
        if ((i & 4) != 0) {
            f2 = noiseEstimate.noiseFloorDb;
        }
        float f5 = f2;
        if ((i & 8) != 0) {
            str = noiseEstimate.state;
        }
        String str2 = str;
        if ((i & 16) != 0) {
            f3 = noiseEstimate.confidence;
        }
        float f6 = f3;
        if ((i & 32) != 0) {
            z = noiseEstimate.changed;
        }
        boolean z2 = z;
        if ((i & 64) != 0) {
            fArr2 = noiseEstimate.bandsDb;
        }
        return noiseEstimate.copy(fArr, f4, f5, str2, f6, z2, fArr2);
    }

    /* renamed from: component1, reason: from getter */
    public final float[] getCleanPower() {
        return this.cleanPower;
    }

    /* renamed from: component2, reason: from getter */
    public final float getSnrDb() {
        return this.snrDb;
    }

    /* renamed from: component3, reason: from getter */
    public final float getNoiseFloorDb() {
        return this.noiseFloorDb;
    }

    /* renamed from: component4, reason: from getter */
    public final String getState() {
        return this.state;
    }

    /* renamed from: component5, reason: from getter */
    public final float getConfidence() {
        return this.confidence;
    }

    /* renamed from: component6, reason: from getter */
    public final boolean getChanged() {
        return this.changed;
    }

    /* renamed from: component7, reason: from getter */
    public final float[] getBandsDb() {
        return this.bandsDb;
    }

    public final NoiseEstimate copy(float[] cleanPower, float snrDb, float noiseFloorDb, String state, float confidence, boolean changed, float[] bandsDb) {
        Intrinsics.checkNotNullParameter(cleanPower, "cleanPower");
        Intrinsics.checkNotNullParameter(state, "state");
        Intrinsics.checkNotNullParameter(bandsDb, "bandsDb");
        return new NoiseEstimate(cleanPower, snrDb, noiseFloorDb, state, confidence, changed, bandsDb);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof NoiseEstimate)) {
            return false;
        }
        NoiseEstimate noiseEstimate = (NoiseEstimate) other;
        return Intrinsics.areEqual(this.cleanPower, noiseEstimate.cleanPower) && Float.compare(this.snrDb, noiseEstimate.snrDb) == 0 && Float.compare(this.noiseFloorDb, noiseEstimate.noiseFloorDb) == 0 && Intrinsics.areEqual(this.state, noiseEstimate.state) && Float.compare(this.confidence, noiseEstimate.confidence) == 0 && this.changed == noiseEstimate.changed && Intrinsics.areEqual(this.bandsDb, noiseEstimate.bandsDb);
    }

    public int hashCode() {
        return (((((((((((Arrays.hashCode(this.cleanPower) * 31) + Float.hashCode(this.snrDb)) * 31) + Float.hashCode(this.noiseFloorDb)) * 31) + this.state.hashCode()) * 31) + Float.hashCode(this.confidence)) * 31) + Boolean.hashCode(this.changed)) * 31) + Arrays.hashCode(this.bandsDb);
    }

    public String toString() {
        return "NoiseEstimate(cleanPower=" + Arrays.toString(this.cleanPower) + ", snrDb=" + this.snrDb + ", noiseFloorDb=" + this.noiseFloorDb + ", state=" + this.state + ", confidence=" + this.confidence + ", changed=" + this.changed + ", bandsDb=" + Arrays.toString(this.bandsDb) + ")";
    }

    public NoiseEstimate(float[] cleanPower, float f, float f2, String state, float f3, boolean z, float[] bandsDb) {
        Intrinsics.checkNotNullParameter(cleanPower, "cleanPower");
        Intrinsics.checkNotNullParameter(state, "state");
        Intrinsics.checkNotNullParameter(bandsDb, "bandsDb");
        this.cleanPower = cleanPower;
        this.snrDb = f;
        this.noiseFloorDb = f2;
        this.state = state;
        this.confidence = f3;
        this.changed = z;
        this.bandsDb = bandsDb;
    }

    public final float[] getCleanPower() {
        return this.cleanPower;
    }

    public final float getSnrDb() {
        return this.snrDb;
    }

    public final float getNoiseFloorDb() {
        return this.noiseFloorDb;
    }

    public final String getState() {
        return this.state;
    }

    public final float getConfidence() {
        return this.confidence;
    }

    public final boolean getChanged() {
        return this.changed;
    }

    public final float[] getBandsDb() {
        return this.bandsDb;
    }
}
