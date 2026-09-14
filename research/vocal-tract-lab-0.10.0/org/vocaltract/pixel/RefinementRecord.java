package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u00004\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\t\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0015\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B9\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0003\u0012\u0006\u0010\u0007\u001a\u00020\b\u0012\u0006\u0010\t\u001a\u00020\n\u0012\b\b\u0002\u0010\u000b\u001a\u00020\u0003¢\u0006\u0004\b\f\u0010\rJ\t\u0010\u0018\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0019\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001a\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001b\u001a\u00020\bHÆ\u0003J\t\u0010\u001c\u001a\u00020\nHÆ\u0003J\t\u0010\u001d\u001a\u00020\u0003HÆ\u0003JE\u0010\u001e\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00032\b\b\u0002\u0010\u0007\u001a\u00020\b2\b\b\u0002\u0010\t\u001a\u00020\n2\b\b\u0002\u0010\u000b\u001a\u00020\u0003HÆ\u0001J\u0013\u0010\u001f\u001a\u00020 2\b\u0010!\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\"\u001a\u00020#HÖ\u0001J\t\u0010$\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000e\u0010\u000fR\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0010\u0010\u0011R\u0011\u0010\u0006\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u000fR\u0011\u0010\u0007\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u0013\u0010\u0014R\u0011\u0010\t\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b\u0015\u0010\u0016R\u0011\u0010\u000b\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0017\u0010\u000f"}, d2 = {"Lorg/vocaltract/pixel/RefinementRecord;", "", "recordId", "", "timestampMillis", "", "sessionId", "observed", "Lorg/vocaltract/pixel/DiagnosticMetrics;", "correction", "Lorg/vocaltract/pixel/RefinementDraft;", "status", "<init>", "(Ljava/lang/String;JLjava/lang/String;Lorg/vocaltract/pixel/DiagnosticMetrics;Lorg/vocaltract/pixel/RefinementDraft;Ljava/lang/String;)V", "getRecordId", "()Ljava/lang/String;", "getTimestampMillis", "()J", "getSessionId", "getObserved", "()Lorg/vocaltract/pixel/DiagnosticMetrics;", "getCorrection", "()Lorg/vocaltract/pixel/RefinementDraft;", "getStatus", "component1", "component2", "component3", "component4", "component5", "component6", "copy", "equals", "", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class RefinementRecord {
    private final RefinementDraft correction;
    private final DiagnosticMetrics observed;
    private final String recordId;
    private final String sessionId;
    private final String status;
    private final long timestampMillis;

    public static /* synthetic */ RefinementRecord copy$default(RefinementRecord refinementRecord, String str, long j, String str2, DiagnosticMetrics diagnosticMetrics, RefinementDraft refinementDraft, String str3, int i, Object obj) {
        if ((i & 1) != 0) {
            str = refinementRecord.recordId;
        }
        if ((i & 2) != 0) {
            j = refinementRecord.timestampMillis;
        }
        long j2 = j;
        if ((i & 4) != 0) {
            str2 = refinementRecord.sessionId;
        }
        String str4 = str2;
        if ((i & 8) != 0) {
            diagnosticMetrics = refinementRecord.observed;
        }
        DiagnosticMetrics diagnosticMetrics2 = diagnosticMetrics;
        if ((i & 16) != 0) {
            refinementDraft = refinementRecord.correction;
        }
        RefinementDraft refinementDraft2 = refinementDraft;
        if ((i & 32) != 0) {
            str3 = refinementRecord.status;
        }
        return refinementRecord.copy(str, j2, str4, diagnosticMetrics2, refinementDraft2, str3);
    }

    /* renamed from: component1, reason: from getter */
    public final String getRecordId() {
        return this.recordId;
    }

    /* renamed from: component2, reason: from getter */
    public final long getTimestampMillis() {
        return this.timestampMillis;
    }

    /* renamed from: component3, reason: from getter */
    public final String getSessionId() {
        return this.sessionId;
    }

    /* renamed from: component4, reason: from getter */
    public final DiagnosticMetrics getObserved() {
        return this.observed;
    }

    /* renamed from: component5, reason: from getter */
    public final RefinementDraft getCorrection() {
        return this.correction;
    }

    /* renamed from: component6, reason: from getter */
    public final String getStatus() {
        return this.status;
    }

    public final RefinementRecord copy(String recordId, long timestampMillis, String sessionId, DiagnosticMetrics observed, RefinementDraft correction, String status) {
        Intrinsics.checkNotNullParameter(recordId, "recordId");
        Intrinsics.checkNotNullParameter(sessionId, "sessionId");
        Intrinsics.checkNotNullParameter(observed, "observed");
        Intrinsics.checkNotNullParameter(correction, "correction");
        Intrinsics.checkNotNullParameter(status, "status");
        return new RefinementRecord(recordId, timestampMillis, sessionId, observed, correction, status);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof RefinementRecord)) {
            return false;
        }
        RefinementRecord refinementRecord = (RefinementRecord) other;
        return Intrinsics.areEqual(this.recordId, refinementRecord.recordId) && this.timestampMillis == refinementRecord.timestampMillis && Intrinsics.areEqual(this.sessionId, refinementRecord.sessionId) && Intrinsics.areEqual(this.observed, refinementRecord.observed) && Intrinsics.areEqual(this.correction, refinementRecord.correction) && Intrinsics.areEqual(this.status, refinementRecord.status);
    }

    public int hashCode() {
        return (((((((((this.recordId.hashCode() * 31) + Long.hashCode(this.timestampMillis)) * 31) + this.sessionId.hashCode()) * 31) + this.observed.hashCode()) * 31) + this.correction.hashCode()) * 31) + this.status.hashCode();
    }

    public String toString() {
        return "RefinementRecord(recordId=" + this.recordId + ", timestampMillis=" + this.timestampMillis + ", sessionId=" + this.sessionId + ", observed=" + this.observed + ", correction=" + this.correction + ", status=" + this.status + ")";
    }

    public RefinementRecord(String recordId, long j, String sessionId, DiagnosticMetrics observed, RefinementDraft correction, String status) {
        Intrinsics.checkNotNullParameter(recordId, "recordId");
        Intrinsics.checkNotNullParameter(sessionId, "sessionId");
        Intrinsics.checkNotNullParameter(observed, "observed");
        Intrinsics.checkNotNullParameter(correction, "correction");
        Intrinsics.checkNotNullParameter(status, "status");
        this.recordId = recordId;
        this.timestampMillis = j;
        this.sessionId = sessionId;
        this.observed = observed;
        this.correction = correction;
        this.status = status;
    }

    public final String getRecordId() {
        return this.recordId;
    }

    public final long getTimestampMillis() {
        return this.timestampMillis;
    }

    public final String getSessionId() {
        return this.sessionId;
    }

    public final DiagnosticMetrics getObserved() {
        return this.observed;
    }

    public final RefinementDraft getCorrection() {
        return this.correction;
    }

    public /* synthetic */ RefinementRecord(String str, long j, String str2, DiagnosticMetrics diagnosticMetrics, RefinementDraft refinementDraft, String str3, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(str, j, str2, diagnosticMetrics, refinementDraft, (i & 32) != 0 ? "proposed" : str3);
    }

    public final String getStatus() {
        return this.status;
    }
}
