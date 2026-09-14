package org.vocaltract.pixel;

import kotlin.Metadata;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u0000\"\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\t\bf\u0018\u00002\u00020\u0001J\u001a\u0010\u0006\u001a\u00020\u00072\u0006\u0010\b\u001a\u00020\t2\b\b\u0002\u0010\n\u001a\u00020\u000bH&R\u0012\u0010\u0002\u001a\u00020\u0003X¦\u0004¢\u0006\u0006\u001a\u0004\b\u0004\u0010\u0005"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticAssistant;", "", "providerId", "", "getProviderId", "()Ljava/lang/String;", "assess", "Lorg/vocaltract/pixel/DiagnosticAssessment;", "metrics", "Lorg/vocaltract/pixel/DiagnosticMetrics;", "nowMillis", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public interface DiagnosticAssistant {
    DiagnosticAssessment assess(DiagnosticMetrics metrics, long nowMillis);

    String getProviderId();

    /* compiled from: DiagnosticsCore.kt */
    @Metadata(k = 3, mv = {2, 0, 0}, xi = 48)
    public static final class DefaultImpls {
        public static /* synthetic */ DiagnosticAssessment assess$default(DiagnosticAssistant diagnosticAssistant, DiagnosticMetrics diagnosticMetrics, long j, int i, Object obj) {
            if (obj != null) {
                throw new UnsupportedOperationException("Super calls with default arguments not supported in this target, function: assess");
            }
            if ((i & 2) != 0) {
                j = System.currentTimeMillis();
            }
            return diagnosticAssistant.assess(diagnosticMetrics, j);
        }
    }
}
