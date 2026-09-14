package org.vocaltract.pixel;

import java.util.List;
import kotlin.Metadata;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u00000\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\t\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0010 \n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0018\b\u0086\b\u0018\u00002\u00020\u0001B?\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0007\u0012\u0006\u0010\b\u001a\u00020\u0003\u0012\f\u0010\t\u001a\b\u0012\u0004\u0012\u00020\u000b0\n\u0012\b\b\u0002\u0010\f\u001a\u00020\r¢\u0006\u0004\b\u000e\u0010\u000fJ\t\u0010\u001b\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001c\u001a\u00020\u0005HÆ\u0003J\t\u0010\u001d\u001a\u00020\u0007HÆ\u0003J\t\u0010\u001e\u001a\u00020\u0003HÆ\u0003J\u000f\u0010\u001f\u001a\b\u0012\u0004\u0012\u00020\u000b0\nHÆ\u0003J\t\u0010 \u001a\u00020\rHÆ\u0003JK\u0010!\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00072\b\b\u0002\u0010\b\u001a\u00020\u00032\u000e\b\u0002\u0010\t\u001a\b\u0012\u0004\u0012\u00020\u000b0\n2\b\b\u0002\u0010\f\u001a\u00020\rHÆ\u0001J\u0013\u0010\"\u001a\u00020\r2\b\u0010#\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010$\u001a\u00020\u0007HÖ\u0001J\t\u0010%\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0010\u0010\u0011R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0013R\u0011\u0010\u0006\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0015R\u0011\u0010\b\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\u0011R\u0017\u0010\t\u001a\b\u0012\u0004\u0012\u00020\u000b0\n¢\u0006\b\n\u0000\u001a\u0004\b\u0017\u0010\u0018R\u0011\u0010\f\u001a\u00020\r¢\u0006\b\n\u0000\u001a\u0004\b\u0019\u0010\u001a"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticAssessment;", "", "providerId", "", "generatedAtMillis", "", "healthScore", "", "summary", "findings", "", "Lorg/vocaltract/pixel/DiagnosticFinding;", "automaticModelMutationAllowed", "", "<init>", "(Ljava/lang/String;JILjava/lang/String;Ljava/util/List;Z)V", "getProviderId", "()Ljava/lang/String;", "getGeneratedAtMillis", "()J", "getHealthScore", "()I", "getSummary", "getFindings", "()Ljava/util/List;", "getAutomaticModelMutationAllowed", "()Z", "component1", "component2", "component3", "component4", "component5", "component6", "copy", "equals", "other", "hashCode", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class DiagnosticAssessment {
    private final boolean automaticModelMutationAllowed;
    private final List<DiagnosticFinding> findings;
    private final long generatedAtMillis;
    private final int healthScore;
    private final String providerId;
    private final String summary;

    public static /* synthetic */ DiagnosticAssessment copy$default(DiagnosticAssessment diagnosticAssessment, String str, long j, int i, String str2, List list, boolean z, int i2, Object obj) {
        if ((i2 & 1) != 0) {
            str = diagnosticAssessment.providerId;
        }
        if ((i2 & 2) != 0) {
            j = diagnosticAssessment.generatedAtMillis;
        }
        long j2 = j;
        if ((i2 & 4) != 0) {
            i = diagnosticAssessment.healthScore;
        }
        int i3 = i;
        if ((i2 & 8) != 0) {
            str2 = diagnosticAssessment.summary;
        }
        String str3 = str2;
        if ((i2 & 16) != 0) {
            list = diagnosticAssessment.findings;
        }
        List list2 = list;
        if ((i2 & 32) != 0) {
            z = diagnosticAssessment.automaticModelMutationAllowed;
        }
        return diagnosticAssessment.copy(str, j2, i3, str3, list2, z);
    }

    /* renamed from: component1, reason: from getter */
    public final String getProviderId() {
        return this.providerId;
    }

    /* renamed from: component2, reason: from getter */
    public final long getGeneratedAtMillis() {
        return this.generatedAtMillis;
    }

    /* renamed from: component3, reason: from getter */
    public final int getHealthScore() {
        return this.healthScore;
    }

    /* renamed from: component4, reason: from getter */
    public final String getSummary() {
        return this.summary;
    }

    public final List<DiagnosticFinding> component5() {
        return this.findings;
    }

    /* renamed from: component6, reason: from getter */
    public final boolean getAutomaticModelMutationAllowed() {
        return this.automaticModelMutationAllowed;
    }

    public final DiagnosticAssessment copy(String providerId, long generatedAtMillis, int healthScore, String summary, List<DiagnosticFinding> findings, boolean automaticModelMutationAllowed) {
        Intrinsics.checkNotNullParameter(providerId, "providerId");
        Intrinsics.checkNotNullParameter(summary, "summary");
        Intrinsics.checkNotNullParameter(findings, "findings");
        return new DiagnosticAssessment(providerId, generatedAtMillis, healthScore, summary, findings, automaticModelMutationAllowed);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof DiagnosticAssessment)) {
            return false;
        }
        DiagnosticAssessment diagnosticAssessment = (DiagnosticAssessment) other;
        return Intrinsics.areEqual(this.providerId, diagnosticAssessment.providerId) && this.generatedAtMillis == diagnosticAssessment.generatedAtMillis && this.healthScore == diagnosticAssessment.healthScore && Intrinsics.areEqual(this.summary, diagnosticAssessment.summary) && Intrinsics.areEqual(this.findings, diagnosticAssessment.findings) && this.automaticModelMutationAllowed == diagnosticAssessment.automaticModelMutationAllowed;
    }

    public int hashCode() {
        return (((((((((this.providerId.hashCode() * 31) + Long.hashCode(this.generatedAtMillis)) * 31) + Integer.hashCode(this.healthScore)) * 31) + this.summary.hashCode()) * 31) + this.findings.hashCode()) * 31) + Boolean.hashCode(this.automaticModelMutationAllowed);
    }

    public String toString() {
        return "DiagnosticAssessment(providerId=" + this.providerId + ", generatedAtMillis=" + this.generatedAtMillis + ", healthScore=" + this.healthScore + ", summary=" + this.summary + ", findings=" + this.findings + ", automaticModelMutationAllowed=" + this.automaticModelMutationAllowed + ")";
    }

    public DiagnosticAssessment(String providerId, long j, int i, String summary, List<DiagnosticFinding> findings, boolean z) {
        Intrinsics.checkNotNullParameter(providerId, "providerId");
        Intrinsics.checkNotNullParameter(summary, "summary");
        Intrinsics.checkNotNullParameter(findings, "findings");
        this.providerId = providerId;
        this.generatedAtMillis = j;
        this.healthScore = i;
        this.summary = summary;
        this.findings = findings;
        this.automaticModelMutationAllowed = z;
    }

    public /* synthetic */ DiagnosticAssessment(String str, long j, int i, String str2, List list, boolean z, int i2, DefaultConstructorMarker defaultConstructorMarker) {
        this(str, j, i, str2, list, (i2 & 32) != 0 ? false : z);
    }

    public final String getProviderId() {
        return this.providerId;
    }

    public final long getGeneratedAtMillis() {
        return this.generatedAtMillis;
    }

    public final int getHealthScore() {
        return this.healthScore;
    }

    public final String getSummary() {
        return this.summary;
    }

    public final List<DiagnosticFinding> getFindings() {
        return this.findings;
    }

    public final boolean getAutomaticModelMutationAllowed() {
        return this.automaticModelMutationAllowed;
    }
}
