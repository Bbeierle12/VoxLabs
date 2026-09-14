package org.vocaltract.pixel;

import java.util.Map;
import kotlin.Metadata;
import kotlin.collections.MapsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u00006\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\t\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010$\n\u0002\b\u0016\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001BM\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0005\u0012\u0006\u0010\u0007\u001a\u00020\u0005\u0012\u0006\u0010\b\u001a\u00020\t\u0012\u0006\u0010\n\u001a\u00020\u0005\u0012\u0014\b\u0002\u0010\u000b\u001a\u000e\u0012\u0004\u0012\u00020\u0005\u0012\u0004\u0012\u00020\u00050\f¢\u0006\u0004\b\r\u0010\u000eJ\t\u0010\u001a\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001b\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001c\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001d\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001e\u001a\u00020\tHÆ\u0003J\t\u0010\u001f\u001a\u00020\u0005HÆ\u0003J\u0015\u0010 \u001a\u000e\u0012\u0004\u0012\u00020\u0005\u0012\u0004\u0012\u00020\u00050\fHÆ\u0003J[\u0010!\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00052\b\b\u0002\u0010\u0007\u001a\u00020\u00052\b\b\u0002\u0010\b\u001a\u00020\t2\b\b\u0002\u0010\n\u001a\u00020\u00052\u0014\b\u0002\u0010\u000b\u001a\u000e\u0012\u0004\u0012\u00020\u0005\u0012\u0004\u0012\u00020\u00050\fHÆ\u0001J\u0013\u0010\"\u001a\u00020#2\b\u0010$\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010%\u001a\u00020&HÖ\u0001J\t\u0010'\u001a\u00020\u0005HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u0010R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0011\u0010\u0012R\u0011\u0010\u0006\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0013\u0010\u0012R\u0011\u0010\u0007\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0012R\u0011\u0010\b\u001a\u00020\t¢\u0006\b\n\u0000\u001a\u0004\b\u0015\u0010\u0016R\u0011\u0010\n\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0017\u0010\u0012R\u001d\u0010\u000b\u001a\u000e\u0012\u0004\u0012\u00020\u0005\u0012\u0004\u0012\u00020\u00050\f¢\u0006\b\n\u0000\u001a\u0004\b\u0018\u0010\u0019"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticEvent;", "", "timestampMillis", "", "sessionId", "", "category", "code", "severity", "Lorg/vocaltract/pixel/DiagnosticSeverity;", "message", "evidence", "", "<init>", "(JLjava/lang/String;Ljava/lang/String;Ljava/lang/String;Lorg/vocaltract/pixel/DiagnosticSeverity;Ljava/lang/String;Ljava/util/Map;)V", "getTimestampMillis", "()J", "getSessionId", "()Ljava/lang/String;", "getCategory", "getCode", "getSeverity", "()Lorg/vocaltract/pixel/DiagnosticSeverity;", "getMessage", "getEvidence", "()Ljava/util/Map;", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "copy", "equals", "", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class DiagnosticEvent {
    private final String category;
    private final String code;
    private final Map<String, String> evidence;
    private final String message;
    private final String sessionId;
    private final DiagnosticSeverity severity;
    private final long timestampMillis;

    /* renamed from: component1, reason: from getter */
    public final long getTimestampMillis() {
        return this.timestampMillis;
    }

    /* renamed from: component2, reason: from getter */
    public final String getSessionId() {
        return this.sessionId;
    }

    /* renamed from: component3, reason: from getter */
    public final String getCategory() {
        return this.category;
    }

    /* renamed from: component4, reason: from getter */
    public final String getCode() {
        return this.code;
    }

    /* renamed from: component5, reason: from getter */
    public final DiagnosticSeverity getSeverity() {
        return this.severity;
    }

    /* renamed from: component6, reason: from getter */
    public final String getMessage() {
        return this.message;
    }

    public final Map<String, String> component7() {
        return this.evidence;
    }

    public final DiagnosticEvent copy(long timestampMillis, String sessionId, String category, String code, DiagnosticSeverity severity, String message, Map<String, String> evidence) {
        Intrinsics.checkNotNullParameter(sessionId, "sessionId");
        Intrinsics.checkNotNullParameter(category, "category");
        Intrinsics.checkNotNullParameter(code, "code");
        Intrinsics.checkNotNullParameter(severity, "severity");
        Intrinsics.checkNotNullParameter(message, "message");
        Intrinsics.checkNotNullParameter(evidence, "evidence");
        return new DiagnosticEvent(timestampMillis, sessionId, category, code, severity, message, evidence);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof DiagnosticEvent)) {
            return false;
        }
        DiagnosticEvent diagnosticEvent = (DiagnosticEvent) other;
        return this.timestampMillis == diagnosticEvent.timestampMillis && Intrinsics.areEqual(this.sessionId, diagnosticEvent.sessionId) && Intrinsics.areEqual(this.category, diagnosticEvent.category) && Intrinsics.areEqual(this.code, diagnosticEvent.code) && this.severity == diagnosticEvent.severity && Intrinsics.areEqual(this.message, diagnosticEvent.message) && Intrinsics.areEqual(this.evidence, diagnosticEvent.evidence);
    }

    public int hashCode() {
        return (((((((((((Long.hashCode(this.timestampMillis) * 31) + this.sessionId.hashCode()) * 31) + this.category.hashCode()) * 31) + this.code.hashCode()) * 31) + this.severity.hashCode()) * 31) + this.message.hashCode()) * 31) + this.evidence.hashCode();
    }

    public String toString() {
        return "DiagnosticEvent(timestampMillis=" + this.timestampMillis + ", sessionId=" + this.sessionId + ", category=" + this.category + ", code=" + this.code + ", severity=" + this.severity + ", message=" + this.message + ", evidence=" + this.evidence + ")";
    }

    public DiagnosticEvent(long j, String sessionId, String category, String code, DiagnosticSeverity severity, String message, Map<String, String> evidence) {
        Intrinsics.checkNotNullParameter(sessionId, "sessionId");
        Intrinsics.checkNotNullParameter(category, "category");
        Intrinsics.checkNotNullParameter(code, "code");
        Intrinsics.checkNotNullParameter(severity, "severity");
        Intrinsics.checkNotNullParameter(message, "message");
        Intrinsics.checkNotNullParameter(evidence, "evidence");
        this.timestampMillis = j;
        this.sessionId = sessionId;
        this.category = category;
        this.code = code;
        this.severity = severity;
        this.message = message;
        this.evidence = evidence;
    }

    public final long getTimestampMillis() {
        return this.timestampMillis;
    }

    public final String getSessionId() {
        return this.sessionId;
    }

    public final String getCategory() {
        return this.category;
    }

    public final String getCode() {
        return this.code;
    }

    public final DiagnosticSeverity getSeverity() {
        return this.severity;
    }

    public final String getMessage() {
        return this.message;
    }

    public /* synthetic */ DiagnosticEvent(long j, String str, String str2, String str3, DiagnosticSeverity diagnosticSeverity, String str4, Map map, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(j, str, str2, str3, diagnosticSeverity, str4, (i & 64) != 0 ? MapsKt.emptyMap() : map);
    }

    public final Map<String, String> getEvidence() {
        return this.evidence;
    }
}
