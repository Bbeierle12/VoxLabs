package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.ranges.RangesKt;

/* compiled from: InferenceStabilizers.kt */
@Metadata(d1 = {"\u0000\"\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u0007\n\u0002\b\u0003\n\u0002\u0010\b\n\u0000\n\u0002\u0018\u0002\n\u0002\b\b\b\u0000\u0018\u00002\u00020\u0001B\u0007¢\u0006\u0004\b\u0002\u0010\u0003J%\u0010\n\u001a\u00020\u000b2\b\u0010\f\u001a\u0004\u0018\u00010\u00052\u0006\u0010\r\u001a\u00020\u00052\u0006\u0010\u000e\u001a\u00020\u0005¢\u0006\u0002\u0010\u000fJ\u001f\u0010\u0010\u001a\u0004\u0018\u00010\u00052\u0006\u0010\u0011\u001a\u00020\u00052\u0006\u0010\u0012\u001a\u00020\u0005H\u0002¢\u0006\u0002\u0010\u0013R\u0012\u0010\u0004\u001a\u0004\u0018\u00010\u0005X\u0082\u000e¢\u0006\u0004\n\u0002\u0010\u0006R\u0012\u0010\u0007\u001a\u0004\u0018\u00010\u0005X\u0082\u000e¢\u0006\u0004\n\u0002\u0010\u0006R\u000e\u0010\b\u001a\u00020\tX\u0082\u000e¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/PitchContinuityGate;", "", "<init>", "()V", "accepted", "", "Ljava/lang/Float;", "pending", "pendingFrames", "", "update", "Lorg/vocaltract/pixel/PitchDecision;", "rawF0Hz", "confidence", "harmonicity", "(Ljava/lang/Float;FF)Lorg/vocaltract/pixel/PitchDecision;", "harmonicFold", "raw", "previous", "(FF)Ljava/lang/Float;"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class PitchContinuityGate {
    private Float accepted;
    private Float pending;
    private int pendingFrames;

    /* JADX WARN: Removed duplicated region for block: B:34:0x00ac  */
    /* JADX WARN: Removed duplicated region for block: B:40:0x00c8  */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public final PitchDecision update(Float rawF0Hz, float confidence, float harmonicity) {
        if (rawF0Hz != null) {
            float floatValue = rawF0Hz.floatValue();
            if (!Float.isInfinite(floatValue) && !Float.isNaN(floatValue)) {
                Float f = this.accepted;
                if (f == null) {
                    this.accepted = rawF0Hz;
                    return new PitchDecision(rawF0Hz, rawF0Hz, "initialized", false);
                }
                float floatValue2 = rawF0Hz.floatValue() / f.floatValue();
                boolean z = floatValue2 > 1.8f || floatValue2 < 0.55f;
                boolean z2 = confidence >= 0.82f && harmonicity >= 0.72f;
                if (!z || !z2) {
                    if (z) {
                        Float harmonicFold = harmonicFold(rawF0Hz.floatValue(), f.floatValue());
                        if (harmonicFold != null) {
                            Float valueOf = Float.valueOf((f.floatValue() * 0.85f) + (harmonicFold.floatValue() * 0.15f));
                            this.accepted = valueOf;
                            this.pending = null;
                            this.pendingFrames = 0;
                            return new PitchDecision(valueOf, rawF0Hz, "harmonic_alias_corrected", true);
                        }
                        this.pending = null;
                        this.pendingFrames = 0;
                        return new PitchDecision(f, rawF0Hz, "large_jump_rejected", true);
                    }
                    this.pending = null;
                    this.pendingFrames = 0;
                    Float valueOf2 = Float.valueOf(f.floatValue() + (RangesKt.coerceIn((RangesKt.coerceIn(confidence, 0.0f, 1.0f) * 0.34f) + 0.18f, 0.18f, 0.52f) * (rawF0Hz.floatValue() - f.floatValue())));
                    this.accepted = valueOf2;
                    return new PitchDecision(valueOf2, rawF0Hz, "tracked", false);
                }
                Float f2 = this.pending;
                if (f2 != null) {
                    float floatValue3 = f2.floatValue();
                    if (Math.abs(rawF0Hz.floatValue() - floatValue3) / Math.max(1.0f, floatValue3) <= 0.08f) {
                        Float f3 = this.pending;
                        if (f3 == null) {
                            throw new IllegalArgumentException("Required value was null.".toString());
                        }
                        this.pending = Float.valueOf((f3.floatValue() * 0.6f) + (rawF0Hz.floatValue() * 0.4f));
                        this.pendingFrames++;
                        if (this.pendingFrames < 3) {
                            Float f4 = this.pending;
                            if (f4 == null) {
                                throw new IllegalArgumentException("Required value was null.".toString());
                            }
                            this.accepted = f4;
                            this.pending = null;
                            this.pendingFrames = 0;
                            return new PitchDecision(f4, rawF0Hz, "persistent_jump_accepted", false);
                        }
                        return new PitchDecision(f, rawF0Hz, "large_jump_pending", true);
                    }
                }
                this.pending = rawF0Hz;
                this.pendingFrames = 1;
                if (this.pendingFrames < 3) {
                }
            }
        }
        this.pending = null;
        this.pendingFrames = 0;
        return new PitchDecision(null, rawF0Hz, "unvoiced", false);
    }

    private final Float harmonicFold(float raw, float previous) {
        float f = Float.POSITIVE_INFINITY;
        Float f2 = null;
        for (int i = 2; i < 9; i++) {
            float f3 = i;
            float[] fArr = {raw / f3, f3 * raw};
            for (int i2 = 0; i2 < 2; i2++) {
                float f4 = fArr[i2];
                float abs = Math.abs(f4 - previous) / Math.max(1.0f, previous);
                if (abs < f) {
                    f2 = Float.valueOf(f4);
                    f = abs;
                }
            }
        }
        if (f2 == null) {
            return null;
        }
        f2.floatValue();
        if (f <= 0.14f) {
            return f2;
        }
        return null;
    }
}
