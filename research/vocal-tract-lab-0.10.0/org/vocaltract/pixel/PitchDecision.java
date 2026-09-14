package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: InferenceStabilizers.kt */
@Metadata(d1 = {"\u0000&\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0002\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0013\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B+\u0012\b\u0010\u0002\u001a\u0004\u0018\u00010\u0003\u0012\b\u0010\u0004\u001a\u0004\u0018\u00010\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0006\u0012\u0006\u0010\u0007\u001a\u00020\b¢\u0006\u0004\b\t\u0010\nJ\u0010\u0010\u0013\u001a\u0004\u0018\u00010\u0003HÆ\u0003¢\u0006\u0002\u0010\fJ\u0010\u0010\u0014\u001a\u0004\u0018\u00010\u0003HÆ\u0003¢\u0006\u0002\u0010\fJ\t\u0010\u0015\u001a\u00020\u0006HÆ\u0003J\t\u0010\u0016\u001a\u00020\bHÆ\u0003J:\u0010\u0017\u001a\u00020\u00002\n\b\u0002\u0010\u0002\u001a\u0004\u0018\u00010\u00032\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u00062\b\b\u0002\u0010\u0007\u001a\u00020\bHÆ\u0001¢\u0006\u0002\u0010\u0018J\u0013\u0010\u0019\u001a\u00020\b2\b\u0010\u001a\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u001b\u001a\u00020\u001cHÖ\u0001J\t\u0010\u001d\u001a\u00020\u0006HÖ\u0001R\u0015\u0010\u0002\u001a\u0004\u0018\u00010\u0003¢\u0006\n\n\u0002\u0010\r\u001a\u0004\b\u000b\u0010\fR\u0015\u0010\u0004\u001a\u0004\u0018\u00010\u0003¢\u0006\n\n\u0002\u0010\r\u001a\u0004\b\u000e\u0010\fR\u0011\u0010\u0005\u001a\u00020\u0006¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u0010R\u0011\u0010\u0007\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u0011\u0010\u0012"}, d2 = {"Lorg/vocaltract/pixel/PitchDecision;", "", "acceptedF0Hz", "", "rawF0Hz", "decision", "", "rejected", "", "<init>", "(Ljava/lang/Float;Ljava/lang/Float;Ljava/lang/String;Z)V", "getAcceptedF0Hz", "()Ljava/lang/Float;", "Ljava/lang/Float;", "getRawF0Hz", "getDecision", "()Ljava/lang/String;", "getRejected", "()Z", "component1", "component2", "component3", "component4", "copy", "(Ljava/lang/Float;Ljava/lang/Float;Ljava/lang/String;Z)Lorg/vocaltract/pixel/PitchDecision;", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class PitchDecision {
    private final Float acceptedF0Hz;
    private final String decision;
    private final Float rawF0Hz;
    private final boolean rejected;

    public static /* synthetic */ PitchDecision copy$default(PitchDecision pitchDecision, Float f, Float f2, String str, boolean z, int i, Object obj) {
        if ((i & 1) != 0) {
            f = pitchDecision.acceptedF0Hz;
        }
        if ((i & 2) != 0) {
            f2 = pitchDecision.rawF0Hz;
        }
        if ((i & 4) != 0) {
            str = pitchDecision.decision;
        }
        if ((i & 8) != 0) {
            z = pitchDecision.rejected;
        }
        return pitchDecision.copy(f, f2, str, z);
    }

    /* renamed from: component1, reason: from getter */
    public final Float getAcceptedF0Hz() {
        return this.acceptedF0Hz;
    }

    /* renamed from: component2, reason: from getter */
    public final Float getRawF0Hz() {
        return this.rawF0Hz;
    }

    /* renamed from: component3, reason: from getter */
    public final String getDecision() {
        return this.decision;
    }

    /* renamed from: component4, reason: from getter */
    public final boolean getRejected() {
        return this.rejected;
    }

    public final PitchDecision copy(Float acceptedF0Hz, Float rawF0Hz, String decision, boolean rejected) {
        Intrinsics.checkNotNullParameter(decision, "decision");
        return new PitchDecision(acceptedF0Hz, rawF0Hz, decision, rejected);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof PitchDecision)) {
            return false;
        }
        PitchDecision pitchDecision = (PitchDecision) other;
        return Intrinsics.areEqual((Object) this.acceptedF0Hz, (Object) pitchDecision.acceptedF0Hz) && Intrinsics.areEqual((Object) this.rawF0Hz, (Object) pitchDecision.rawF0Hz) && Intrinsics.areEqual(this.decision, pitchDecision.decision) && this.rejected == pitchDecision.rejected;
    }

    public int hashCode() {
        Float f = this.acceptedF0Hz;
        int hashCode = (f == null ? 0 : f.hashCode()) * 31;
        Float f2 = this.rawF0Hz;
        return ((((hashCode + (f2 != null ? f2.hashCode() : 0)) * 31) + this.decision.hashCode()) * 31) + Boolean.hashCode(this.rejected);
    }

    public String toString() {
        return "PitchDecision(acceptedF0Hz=" + this.acceptedF0Hz + ", rawF0Hz=" + this.rawF0Hz + ", decision=" + this.decision + ", rejected=" + this.rejected + ")";
    }

    public PitchDecision(Float f, Float f2, String decision, boolean z) {
        Intrinsics.checkNotNullParameter(decision, "decision");
        this.acceptedF0Hz = f;
        this.rawF0Hz = f2;
        this.decision = decision;
        this.rejected = z;
    }

    public final Float getAcceptedF0Hz() {
        return this.acceptedF0Hz;
    }

    public final Float getRawF0Hz() {
        return this.rawF0Hz;
    }

    public final String getDecision() {
        return this.decision;
    }

    public final boolean getRejected() {
        return this.rejected;
    }
}
