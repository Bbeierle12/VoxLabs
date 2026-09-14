package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: DiagnosticStore.kt */
@Metadata(d1 = {"\u0000\u001e\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u000f\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B\u001f\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0003¢\u0006\u0004\b\u0007\u0010\bJ\t\u0010\u000e\u001a\u00020\u0003HÆ\u0003J\t\u0010\u000f\u001a\u00020\u0005HÆ\u0003J\t\u0010\u0010\u001a\u00020\u0003HÆ\u0003J'\u0010\u0011\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u0003HÆ\u0001J\u0013\u0010\u0012\u001a\u00020\u00052\b\u0010\u0013\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0014\u001a\u00020\u0015HÖ\u0001J\t\u0010\u0016\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\t\u0010\nR\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\fR\u0011\u0010\u0006\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\r\u0010\n"}, d2 = {"Lorg/vocaltract/pixel/SelfTestResult;", "", "code", "", "passed", "", "message", "<init>", "(Ljava/lang/String;ZLjava/lang/String;)V", "getCode", "()Ljava/lang/String;", "getPassed", "()Z", "getMessage", "component1", "component2", "component3", "copy", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class SelfTestResult {
    private final String code;
    private final String message;
    private final boolean passed;

    public static /* synthetic */ SelfTestResult copy$default(SelfTestResult selfTestResult, String str, boolean z, String str2, int i, Object obj) {
        if ((i & 1) != 0) {
            str = selfTestResult.code;
        }
        if ((i & 2) != 0) {
            z = selfTestResult.passed;
        }
        if ((i & 4) != 0) {
            str2 = selfTestResult.message;
        }
        return selfTestResult.copy(str, z, str2);
    }

    /* renamed from: component1, reason: from getter */
    public final String getCode() {
        return this.code;
    }

    /* renamed from: component2, reason: from getter */
    public final boolean getPassed() {
        return this.passed;
    }

    /* renamed from: component3, reason: from getter */
    public final String getMessage() {
        return this.message;
    }

    public final SelfTestResult copy(String code, boolean passed, String message) {
        Intrinsics.checkNotNullParameter(code, "code");
        Intrinsics.checkNotNullParameter(message, "message");
        return new SelfTestResult(code, passed, message);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof SelfTestResult)) {
            return false;
        }
        SelfTestResult selfTestResult = (SelfTestResult) other;
        return Intrinsics.areEqual(this.code, selfTestResult.code) && this.passed == selfTestResult.passed && Intrinsics.areEqual(this.message, selfTestResult.message);
    }

    public int hashCode() {
        return (((this.code.hashCode() * 31) + Boolean.hashCode(this.passed)) * 31) + this.message.hashCode();
    }

    public String toString() {
        return "SelfTestResult(code=" + this.code + ", passed=" + this.passed + ", message=" + this.message + ")";
    }

    public SelfTestResult(String code, boolean z, String message) {
        Intrinsics.checkNotNullParameter(code, "code");
        Intrinsics.checkNotNullParameter(message, "message");
        this.code = code;
        this.passed = z;
        this.message = message;
    }

    public final String getCode() {
        return this.code;
    }

    public final boolean getPassed() {
        return this.passed;
    }

    public final String getMessage() {
        return this.message;
    }
}
