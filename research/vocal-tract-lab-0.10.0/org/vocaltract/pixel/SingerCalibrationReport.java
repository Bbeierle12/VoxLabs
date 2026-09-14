package org.vocaltract.pixel;

import java.util.List;
import kotlin.Metadata;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: SingerCalibration.kt */
@Metadata(d1 = {"\u00000\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\t\n\u0002\b\u0002\n\u0002\u0010 \n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0017\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001BC\u0012\b\b\u0002\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0005\u0012\f\u0010\u0007\u001a\b\u0012\u0004\u0012\u00020\t0\b\u0012\b\b\u0002\u0010\n\u001a\u00020\u000b\u0012\b\b\u0002\u0010\f\u001a\u00020\u000b¢\u0006\u0004\b\r\u0010\u000eJ\t\u0010\u0019\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001a\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001b\u001a\u00020\u0005HÆ\u0003J\u000f\u0010\u001c\u001a\b\u0012\u0004\u0012\u00020\t0\bHÆ\u0003J\t\u0010\u001d\u001a\u00020\u000bHÆ\u0003J\t\u0010\u001e\u001a\u00020\u000bHÆ\u0003JK\u0010\u001f\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00052\u000e\b\u0002\u0010\u0007\u001a\b\u0012\u0004\u0012\u00020\t0\b2\b\b\u0002\u0010\n\u001a\u00020\u000b2\b\b\u0002\u0010\f\u001a\u00020\u000bHÆ\u0001J\u0013\u0010 \u001a\u00020\u000b2\b\u0010!\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\"\u001a\u00020#HÖ\u0001J\t\u0010$\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u0010R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0011\u0010\u0012R\u0011\u0010\u0006\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0013\u0010\u0012R\u0017\u0010\u0007\u001a\b\u0012\u0004\u0012\u00020\t0\b¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0015R\u0011\u0010\n\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\u0017R\u0011\u0010\f\u001a\u00020\u000b¢\u0006\b\n\u0000\u001a\u0004\b\u0018\u0010\u0017"}, d2 = {"Lorg/vocaltract/pixel/SingerCalibrationReport;", "", "schemaVersion", "", "startedAtMillis", "", "completedAtMillis", "steps", "", "Lorg/vocaltract/pixel/CalibrationStepSummary;", "rawAudioStored", "", "automaticModelMutation", "<init>", "(Ljava/lang/String;JJLjava/util/List;ZZ)V", "getSchemaVersion", "()Ljava/lang/String;", "getStartedAtMillis", "()J", "getCompletedAtMillis", "getSteps", "()Ljava/util/List;", "getRawAudioStored", "()Z", "getAutomaticModelMutation", "component1", "component2", "component3", "component4", "component5", "component6", "copy", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class SingerCalibrationReport {
    private final boolean automaticModelMutation;
    private final long completedAtMillis;
    private final boolean rawAudioStored;
    private final String schemaVersion;
    private final long startedAtMillis;
    private final List<CalibrationStepSummary> steps;

    /* renamed from: component1, reason: from getter */
    public final String getSchemaVersion() {
        return this.schemaVersion;
    }

    /* renamed from: component2, reason: from getter */
    public final long getStartedAtMillis() {
        return this.startedAtMillis;
    }

    /* renamed from: component3, reason: from getter */
    public final long getCompletedAtMillis() {
        return this.completedAtMillis;
    }

    public final List<CalibrationStepSummary> component4() {
        return this.steps;
    }

    /* renamed from: component5, reason: from getter */
    public final boolean getRawAudioStored() {
        return this.rawAudioStored;
    }

    /* renamed from: component6, reason: from getter */
    public final boolean getAutomaticModelMutation() {
        return this.automaticModelMutation;
    }

    public final SingerCalibrationReport copy(String schemaVersion, long startedAtMillis, long completedAtMillis, List<CalibrationStepSummary> steps, boolean rawAudioStored, boolean automaticModelMutation) {
        Intrinsics.checkNotNullParameter(schemaVersion, "schemaVersion");
        Intrinsics.checkNotNullParameter(steps, "steps");
        return new SingerCalibrationReport(schemaVersion, startedAtMillis, completedAtMillis, steps, rawAudioStored, automaticModelMutation);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof SingerCalibrationReport)) {
            return false;
        }
        SingerCalibrationReport singerCalibrationReport = (SingerCalibrationReport) other;
        return Intrinsics.areEqual(this.schemaVersion, singerCalibrationReport.schemaVersion) && this.startedAtMillis == singerCalibrationReport.startedAtMillis && this.completedAtMillis == singerCalibrationReport.completedAtMillis && Intrinsics.areEqual(this.steps, singerCalibrationReport.steps) && this.rawAudioStored == singerCalibrationReport.rawAudioStored && this.automaticModelMutation == singerCalibrationReport.automaticModelMutation;
    }

    public int hashCode() {
        return (((((((((this.schemaVersion.hashCode() * 31) + Long.hashCode(this.startedAtMillis)) * 31) + Long.hashCode(this.completedAtMillis)) * 31) + this.steps.hashCode()) * 31) + Boolean.hashCode(this.rawAudioStored)) * 31) + Boolean.hashCode(this.automaticModelMutation);
    }

    public String toString() {
        return "SingerCalibrationReport(schemaVersion=" + this.schemaVersion + ", startedAtMillis=" + this.startedAtMillis + ", completedAtMillis=" + this.completedAtMillis + ", steps=" + this.steps + ", rawAudioStored=" + this.rawAudioStored + ", automaticModelMutation=" + this.automaticModelMutation + ")";
    }

    public SingerCalibrationReport(String schemaVersion, long j, long j2, List<CalibrationStepSummary> steps, boolean z, boolean z2) {
        Intrinsics.checkNotNullParameter(schemaVersion, "schemaVersion");
        Intrinsics.checkNotNullParameter(steps, "steps");
        this.schemaVersion = schemaVersion;
        this.startedAtMillis = j;
        this.completedAtMillis = j2;
        this.steps = steps;
        this.rawAudioStored = z;
        this.automaticModelMutation = z2;
    }

    public /* synthetic */ SingerCalibrationReport(String str, long j, long j2, List list, boolean z, boolean z2, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this((i & 1) != 0 ? "vocaltract3d.singer-calibration/1.0" : str, j, j2, list, (i & 16) != 0 ? false : z, (i & 32) != 0 ? false : z2);
    }

    public final String getSchemaVersion() {
        return this.schemaVersion;
    }

    public final long getStartedAtMillis() {
        return this.startedAtMillis;
    }

    public final long getCompletedAtMillis() {
        return this.completedAtMillis;
    }

    public final List<CalibrationStepSummary> getSteps() {
        return this.steps;
    }

    public final boolean getRawAudioStored() {
        return this.rawAudioStored;
    }

    public final boolean getAutomaticModelMutation() {
        return this.automaticModelMutation;
    }
}
