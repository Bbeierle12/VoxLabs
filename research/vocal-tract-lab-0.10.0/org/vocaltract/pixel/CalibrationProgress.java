package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: SingerCalibration.kt */
@Metadata(d1 = {"\u0000,\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000b\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\u0007\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0018\b\u0086\b\u0018\u00002\u00020\u0001B;\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0005\u0012\u0006\u0010\u0007\u001a\u00020\b\u0012\u0006\u0010\t\u001a\u00020\n\u0012\n\b\u0002\u0010\u000b\u001a\u0004\u0018\u00010\f¢\u0006\u0004\b\r\u0010\u000eJ\t\u0010\u001a\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001b\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001c\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001d\u001a\u00020\bHÆ\u0003J\t\u0010\u001e\u001a\u00020\nHÆ\u0003J\u000b\u0010\u001f\u001a\u0004\u0018\u00010\fHÆ\u0003JG\u0010 \u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00052\b\b\u0002\u0010\u0007\u001a\u00020\b2\b\b\u0002\u0010\t\u001a\u00020\n2\n\b\u0002\u0010\u000b\u001a\u0004\u0018\u00010\fHÆ\u0001J\u0013\u0010!\u001a\u00020\u00032\b\u0010\"\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010#\u001a\u00020\u0005HÖ\u0001J\t\u0010$\u001a\u00020\bHÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u0010R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0011\u0010\u0012R\u0011\u0010\u0006\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0013\u0010\u0012R\u0011\u0010\u0007\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0015R\u0011\u0010\t\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\u0017R\u0013\u0010\u000b\u001a\u0004\u0018\u00010\f¢\u0006\b\n\u0000\u001a\u0004\b\u0018\u0010\u0019"}, d2 = {"Lorg/vocaltract/pixel/CalibrationProgress;", "", "active", "", "stepIndex", "", "stepCount", "prompt", "", "fraction", "", "completedReport", "Lorg/vocaltract/pixel/SingerCalibrationReport;", "<init>", "(ZIILjava/lang/String;FLorg/vocaltract/pixel/SingerCalibrationReport;)V", "getActive", "()Z", "getStepIndex", "()I", "getStepCount", "getPrompt", "()Ljava/lang/String;", "getFraction", "()F", "getCompletedReport", "()Lorg/vocaltract/pixel/SingerCalibrationReport;", "component1", "component2", "component3", "component4", "component5", "component6", "copy", "equals", "other", "hashCode", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class CalibrationProgress {
    private final boolean active;
    private final SingerCalibrationReport completedReport;
    private final float fraction;
    private final String prompt;
    private final int stepCount;
    private final int stepIndex;

    public static /* synthetic */ CalibrationProgress copy$default(CalibrationProgress calibrationProgress, boolean z, int i, int i2, String str, float f, SingerCalibrationReport singerCalibrationReport, int i3, Object obj) {
        if ((i3 & 1) != 0) {
            z = calibrationProgress.active;
        }
        if ((i3 & 2) != 0) {
            i = calibrationProgress.stepIndex;
        }
        int i4 = i;
        if ((i3 & 4) != 0) {
            i2 = calibrationProgress.stepCount;
        }
        int i5 = i2;
        if ((i3 & 8) != 0) {
            str = calibrationProgress.prompt;
        }
        String str2 = str;
        if ((i3 & 16) != 0) {
            f = calibrationProgress.fraction;
        }
        float f2 = f;
        if ((i3 & 32) != 0) {
            singerCalibrationReport = calibrationProgress.completedReport;
        }
        return calibrationProgress.copy(z, i4, i5, str2, f2, singerCalibrationReport);
    }

    /* renamed from: component1, reason: from getter */
    public final boolean getActive() {
        return this.active;
    }

    /* renamed from: component2, reason: from getter */
    public final int getStepIndex() {
        return this.stepIndex;
    }

    /* renamed from: component3, reason: from getter */
    public final int getStepCount() {
        return this.stepCount;
    }

    /* renamed from: component4, reason: from getter */
    public final String getPrompt() {
        return this.prompt;
    }

    /* renamed from: component5, reason: from getter */
    public final float getFraction() {
        return this.fraction;
    }

    /* renamed from: component6, reason: from getter */
    public final SingerCalibrationReport getCompletedReport() {
        return this.completedReport;
    }

    public final CalibrationProgress copy(boolean active, int stepIndex, int stepCount, String prompt, float fraction, SingerCalibrationReport completedReport) {
        Intrinsics.checkNotNullParameter(prompt, "prompt");
        return new CalibrationProgress(active, stepIndex, stepCount, prompt, fraction, completedReport);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CalibrationProgress)) {
            return false;
        }
        CalibrationProgress calibrationProgress = (CalibrationProgress) other;
        return this.active == calibrationProgress.active && this.stepIndex == calibrationProgress.stepIndex && this.stepCount == calibrationProgress.stepCount && Intrinsics.areEqual(this.prompt, calibrationProgress.prompt) && Float.compare(this.fraction, calibrationProgress.fraction) == 0 && Intrinsics.areEqual(this.completedReport, calibrationProgress.completedReport);
    }

    public int hashCode() {
        int hashCode = ((((((((Boolean.hashCode(this.active) * 31) + Integer.hashCode(this.stepIndex)) * 31) + Integer.hashCode(this.stepCount)) * 31) + this.prompt.hashCode()) * 31) + Float.hashCode(this.fraction)) * 31;
        SingerCalibrationReport singerCalibrationReport = this.completedReport;
        return hashCode + (singerCalibrationReport == null ? 0 : singerCalibrationReport.hashCode());
    }

    public String toString() {
        return "CalibrationProgress(active=" + this.active + ", stepIndex=" + this.stepIndex + ", stepCount=" + this.stepCount + ", prompt=" + this.prompt + ", fraction=" + this.fraction + ", completedReport=" + this.completedReport + ")";
    }

    public CalibrationProgress(boolean z, int i, int i2, String prompt, float f, SingerCalibrationReport singerCalibrationReport) {
        Intrinsics.checkNotNullParameter(prompt, "prompt");
        this.active = z;
        this.stepIndex = i;
        this.stepCount = i2;
        this.prompt = prompt;
        this.fraction = f;
        this.completedReport = singerCalibrationReport;
    }

    public /* synthetic */ CalibrationProgress(boolean z, int i, int i2, String str, float f, SingerCalibrationReport singerCalibrationReport, int i3, DefaultConstructorMarker defaultConstructorMarker) {
        this(z, i, i2, str, f, (i3 & 32) != 0 ? null : singerCalibrationReport);
    }

    public final boolean getActive() {
        return this.active;
    }

    public final int getStepIndex() {
        return this.stepIndex;
    }

    public final int getStepCount() {
        return this.stepCount;
    }

    public final String getPrompt() {
        return this.prompt;
    }

    public final float getFraction() {
        return this.fraction;
    }

    public final SingerCalibrationReport getCompletedReport() {
        return this.completedReport;
    }
}
