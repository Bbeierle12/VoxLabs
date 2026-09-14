package org.vocaltract.pixel;

import java.util.Arrays;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: SharedTractModel.kt */
@Metadata(d1 = {"\u0000*\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0014\n\u0002\b\u0003\n\u0002\u0010\u0007\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0013\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u000e\b\u0086\b\u0018\u00002\u00020\u0001B/\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0003\u0012\u0006\u0010\u0006\u001a\u00020\u0007\u0012\u0006\u0010\b\u001a\u00020\t¢\u0006\u0004\b\n\u0010\u000bJ\t\u0010\u0014\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0015\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0016\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0017\u001a\u00020\u0007HÆ\u0003J\t\u0010\u0018\u001a\u00020\tHÆ\u0003J;\u0010\u0019\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u00032\b\b\u0002\u0010\u0006\u001a\u00020\u00072\b\b\u0002\u0010\b\u001a\u00020\tHÆ\u0001J\u0013\u0010\u001a\u001a\u00020\t2\b\u0010\u001b\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u001c\u001a\u00020\u001dHÖ\u0001J\t\u0010\u001e\u001a\u00020\u001fHÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\f\u0010\rR\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000e\u0010\rR\u0011\u0010\u0005\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\rR\u0011\u0010\u0006\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u0010\u0010\u0011R\u0011\u0010\b\u001a\u00020\t¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0013"}, d2 = {"Lorg/vocaltract/pixel/TractFit;", "", "coefficients", "", "targets", "achieved", "rmseHz", "", "closeMatch", "", "<init>", "([F[F[FFZ)V", "getCoefficients", "()[F", "getTargets", "getAchieved", "getRmseHz", "()F", "getCloseMatch", "()Z", "component1", "component2", "component3", "component4", "component5", "copy", "equals", "other", "hashCode", "", "toString", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class TractFit {
    private final float[] achieved;
    private final boolean closeMatch;
    private final float[] coefficients;
    private final float rmseHz;
    private final float[] targets;

    public static /* synthetic */ TractFit copy$default(TractFit tractFit, float[] fArr, float[] fArr2, float[] fArr3, float f, boolean z, int i, Object obj) {
        if ((i & 1) != 0) {
            fArr = tractFit.coefficients;
        }
        if ((i & 2) != 0) {
            fArr2 = tractFit.targets;
        }
        float[] fArr4 = fArr2;
        if ((i & 4) != 0) {
            fArr3 = tractFit.achieved;
        }
        float[] fArr5 = fArr3;
        if ((i & 8) != 0) {
            f = tractFit.rmseHz;
        }
        float f2 = f;
        if ((i & 16) != 0) {
            z = tractFit.closeMatch;
        }
        return tractFit.copy(fArr, fArr4, fArr5, f2, z);
    }

    /* renamed from: component1, reason: from getter */
    public final float[] getCoefficients() {
        return this.coefficients;
    }

    /* renamed from: component2, reason: from getter */
    public final float[] getTargets() {
        return this.targets;
    }

    /* renamed from: component3, reason: from getter */
    public final float[] getAchieved() {
        return this.achieved;
    }

    /* renamed from: component4, reason: from getter */
    public final float getRmseHz() {
        return this.rmseHz;
    }

    /* renamed from: component5, reason: from getter */
    public final boolean getCloseMatch() {
        return this.closeMatch;
    }

    public final TractFit copy(float[] coefficients, float[] targets, float[] achieved, float rmseHz, boolean closeMatch) {
        Intrinsics.checkNotNullParameter(coefficients, "coefficients");
        Intrinsics.checkNotNullParameter(targets, "targets");
        Intrinsics.checkNotNullParameter(achieved, "achieved");
        return new TractFit(coefficients, targets, achieved, rmseHz, closeMatch);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof TractFit)) {
            return false;
        }
        TractFit tractFit = (TractFit) other;
        return Intrinsics.areEqual(this.coefficients, tractFit.coefficients) && Intrinsics.areEqual(this.targets, tractFit.targets) && Intrinsics.areEqual(this.achieved, tractFit.achieved) && Float.compare(this.rmseHz, tractFit.rmseHz) == 0 && this.closeMatch == tractFit.closeMatch;
    }

    public int hashCode() {
        return (((((((Arrays.hashCode(this.coefficients) * 31) + Arrays.hashCode(this.targets)) * 31) + Arrays.hashCode(this.achieved)) * 31) + Float.hashCode(this.rmseHz)) * 31) + Boolean.hashCode(this.closeMatch);
    }

    public String toString() {
        return "TractFit(coefficients=" + Arrays.toString(this.coefficients) + ", targets=" + Arrays.toString(this.targets) + ", achieved=" + Arrays.toString(this.achieved) + ", rmseHz=" + this.rmseHz + ", closeMatch=" + this.closeMatch + ")";
    }

    public TractFit(float[] coefficients, float[] targets, float[] achieved, float f, boolean z) {
        Intrinsics.checkNotNullParameter(coefficients, "coefficients");
        Intrinsics.checkNotNullParameter(targets, "targets");
        Intrinsics.checkNotNullParameter(achieved, "achieved");
        this.coefficients = coefficients;
        this.targets = targets;
        this.achieved = achieved;
        this.rmseHz = f;
        this.closeMatch = z;
    }

    public final float[] getAchieved() {
        return this.achieved;
    }

    public final boolean getCloseMatch() {
        return this.closeMatch;
    }

    public final float[] getCoefficients() {
        return this.coefficients;
    }

    public final float getRmseHz() {
        return this.rmseHz;
    }

    public final float[] getTargets() {
        return this.targets;
    }
}
