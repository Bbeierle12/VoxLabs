package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u0000.\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0010\u0007\n\u0002\b\u0013\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B7\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0003\u0012\u0006\u0010\u0007\u001a\u00020\u0003\u0012\u0006\u0010\b\u001a\u00020\u0003\u0012\u0006\u0010\t\u001a\u00020\n¢\u0006\u0004\b\u000b\u0010\fJ\t\u0010\u0016\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0017\u001a\u00020\u0005HÆ\u0003J\t\u0010\u0018\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0019\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001a\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001b\u001a\u00020\nHÆ\u0003JE\u0010\u001c\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00032\b\b\u0002\u0010\u0007\u001a\u00020\u00032\b\b\u0002\u0010\b\u001a\u00020\u00032\b\b\u0002\u0010\t\u001a\u00020\nHÆ\u0001J\u0013\u0010\u001d\u001a\u00020\u001e2\b\u0010\u001f\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010 \u001a\u00020!HÖ\u0001J\t\u0010\"\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\r\u0010\u000eR\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u0010R\u0011\u0010\u0006\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0011\u0010\u000eR\u0011\u0010\u0007\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u000eR\u0011\u0010\b\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0013\u0010\u000eR\u0011\u0010\t\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0015"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticFinding;", "", "code", "", "severity", "Lorg/vocaltract/pixel/DiagnosticSeverity;", "title", "evidence", "recommendedAction", "confidence", "", "<init>", "(Ljava/lang/String;Lorg/vocaltract/pixel/DiagnosticSeverity;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;F)V", "getCode", "()Ljava/lang/String;", "getSeverity", "()Lorg/vocaltract/pixel/DiagnosticSeverity;", "getTitle", "getEvidence", "getRecommendedAction", "getConfidence", "()F", "component1", "component2", "component3", "component4", "component5", "component6", "copy", "equals", "", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class DiagnosticFinding {
    private final String code;
    private final float confidence;
    private final String evidence;
    private final String recommendedAction;
    private final DiagnosticSeverity severity;
    private final String title;

    public static /* synthetic */ DiagnosticFinding copy$default(DiagnosticFinding diagnosticFinding, String str, DiagnosticSeverity diagnosticSeverity, String str2, String str3, String str4, float f, int i, Object obj) {
        if ((i & 1) != 0) {
            str = diagnosticFinding.code;
        }
        if ((i & 2) != 0) {
            diagnosticSeverity = diagnosticFinding.severity;
        }
        DiagnosticSeverity diagnosticSeverity2 = diagnosticSeverity;
        if ((i & 4) != 0) {
            str2 = diagnosticFinding.title;
        }
        String str5 = str2;
        if ((i & 8) != 0) {
            str3 = diagnosticFinding.evidence;
        }
        String str6 = str3;
        if ((i & 16) != 0) {
            str4 = diagnosticFinding.recommendedAction;
        }
        String str7 = str4;
        if ((i & 32) != 0) {
            f = diagnosticFinding.confidence;
        }
        return diagnosticFinding.copy(str, diagnosticSeverity2, str5, str6, str7, f);
    }

    /* renamed from: component1, reason: from getter */
    public final String getCode() {
        return this.code;
    }

    /* renamed from: component2, reason: from getter */
    public final DiagnosticSeverity getSeverity() {
        return this.severity;
    }

    /* renamed from: component3, reason: from getter */
    public final String getTitle() {
        return this.title;
    }

    /* renamed from: component4, reason: from getter */
    public final String getEvidence() {
        return this.evidence;
    }

    /* renamed from: component5, reason: from getter */
    public final String getRecommendedAction() {
        return this.recommendedAction;
    }

    /* renamed from: component6, reason: from getter */
    public final float getConfidence() {
        return this.confidence;
    }

    public final DiagnosticFinding copy(String code, DiagnosticSeverity severity, String title, String evidence, String recommendedAction, float confidence) {
        Intrinsics.checkNotNullParameter(code, "code");
        Intrinsics.checkNotNullParameter(severity, "severity");
        Intrinsics.checkNotNullParameter(title, "title");
        Intrinsics.checkNotNullParameter(evidence, "evidence");
        Intrinsics.checkNotNullParameter(recommendedAction, "recommendedAction");
        return new DiagnosticFinding(code, severity, title, evidence, recommendedAction, confidence);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof DiagnosticFinding)) {
            return false;
        }
        DiagnosticFinding diagnosticFinding = (DiagnosticFinding) other;
        return Intrinsics.areEqual(this.code, diagnosticFinding.code) && this.severity == diagnosticFinding.severity && Intrinsics.areEqual(this.title, diagnosticFinding.title) && Intrinsics.areEqual(this.evidence, diagnosticFinding.evidence) && Intrinsics.areEqual(this.recommendedAction, diagnosticFinding.recommendedAction) && Float.compare(this.confidence, diagnosticFinding.confidence) == 0;
    }

    public int hashCode() {
        return (((((((((this.code.hashCode() * 31) + this.severity.hashCode()) * 31) + this.title.hashCode()) * 31) + this.evidence.hashCode()) * 31) + this.recommendedAction.hashCode()) * 31) + Float.hashCode(this.confidence);
    }

    public String toString() {
        return "DiagnosticFinding(code=" + this.code + ", severity=" + this.severity + ", title=" + this.title + ", evidence=" + this.evidence + ", recommendedAction=" + this.recommendedAction + ", confidence=" + this.confidence + ")";
    }

    public DiagnosticFinding(String code, DiagnosticSeverity severity, String title, String evidence, String recommendedAction, float f) {
        Intrinsics.checkNotNullParameter(code, "code");
        Intrinsics.checkNotNullParameter(severity, "severity");
        Intrinsics.checkNotNullParameter(title, "title");
        Intrinsics.checkNotNullParameter(evidence, "evidence");
        Intrinsics.checkNotNullParameter(recommendedAction, "recommendedAction");
        this.code = code;
        this.severity = severity;
        this.title = title;
        this.evidence = evidence;
        this.recommendedAction = recommendedAction;
        this.confidence = f;
    }

    public final String getCode() {
        return this.code;
    }

    public final DiagnosticSeverity getSeverity() {
        return this.severity;
    }

    public final String getTitle() {
        return this.title;
    }

    public final String getEvidence() {
        return this.evidence;
    }

    public final String getRecommendedAction() {
        return this.recommendedAction;
    }

    public final float getConfidence() {
        return this.confidence;
    }
}
