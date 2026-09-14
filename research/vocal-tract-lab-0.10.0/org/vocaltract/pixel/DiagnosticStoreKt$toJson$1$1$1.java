package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.Unit;
import kotlin.jvm.functions.Function2;
import kotlin.jvm.internal.AdaptedFunctionReference;
import org.json.JSONObject;

/* compiled from: DiagnosticStore.kt */
@Metadata(k = 3, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
/* synthetic */ class DiagnosticStoreKt$toJson$1$1$1 extends AdaptedFunctionReference implements Function2<String, Object, Unit> {
    DiagnosticStoreKt$toJson$1$1$1(Object obj) {
        super(2, obj, JSONObject.class, "put", "put(Ljava/lang/String;Ljava/lang/Object;)Lorg/json/JSONObject;", 8);
    }

    /* JADX DEBUG: Method arguments types fixed to match base method, original types: [java.lang.Object, java.lang.Object] */
    /* JADX DEBUG: Return type fixed from 'java.lang.Object' to match base method */
    @Override // kotlin.jvm.functions.Function2
    public /* bridge */ /* synthetic */ Unit invoke(String str, Object obj) {
        invoke2(str, obj);
        return Unit.INSTANCE;
    }

    /* renamed from: invoke, reason: avoid collision after fix types in other method */
    public final void invoke2(String str, Object obj) {
        ((JSONObject) this.receiver).put(str, obj);
    }
}
