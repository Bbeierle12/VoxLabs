package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.enums.EnumEntries;
import kotlin.enums.EnumEntriesKt;

/* JADX WARN: Failed to restore enum class, 'enum' modifier and super class removed */
/* JADX WARN: Unknown enum class pattern. Please report as an issue! */
/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u0000\f\n\u0002\u0018\u0002\n\u0002\u0010\u0010\n\u0002\b\u0005\b\u0086\u0081\u0002\u0018\u00002\b\u0012\u0004\u0012\u00020\u00000\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003j\u0002\b\u0004j\u0002\b\u0005j\u0002\b\u0006"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticSeverity;", "", "<init>", "(Ljava/lang/String;I)V", "INFO", "WARNING", "ERROR"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class DiagnosticSeverity {
    private static final /* synthetic */ EnumEntries $ENTRIES;
    private static final /* synthetic */ DiagnosticSeverity[] $VALUES;
    public static final DiagnosticSeverity INFO = new DiagnosticSeverity("INFO", 0);
    public static final DiagnosticSeverity WARNING = new DiagnosticSeverity("WARNING", 1);
    public static final DiagnosticSeverity ERROR = new DiagnosticSeverity("ERROR", 2);

    private static final /* synthetic */ DiagnosticSeverity[] $values() {
        return new DiagnosticSeverity[]{INFO, WARNING, ERROR};
    }

    public static EnumEntries<DiagnosticSeverity> getEntries() {
        return $ENTRIES;
    }

    private DiagnosticSeverity(String str, int i) {
    }

    static {
        DiagnosticSeverity[] $values = $values();
        $VALUES = $values;
        $ENTRIES = EnumEntriesKt.enumEntries($values);
    }

    public static DiagnosticSeverity valueOf(String str) {
        return (DiagnosticSeverity) Enum.valueOf(DiagnosticSeverity.class, str);
    }

    public static DiagnosticSeverity[] values() {
        return (DiagnosticSeverity[]) $VALUES.clone();
    }
}
