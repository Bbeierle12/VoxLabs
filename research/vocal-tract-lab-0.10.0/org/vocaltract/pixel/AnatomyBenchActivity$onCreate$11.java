package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.Unit;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.FunctionReferenceImpl;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: AnatomyBenchActivity.kt */
@Metadata(k = 3, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
/* synthetic */ class AnatomyBenchActivity$onCreate$11 extends FunctionReferenceImpl implements Function1<SingerCalibrationReport, Unit> {
    AnatomyBenchActivity$onCreate$11(Object obj) {
        super(1, obj, DiagnosticsRuntime.class, "recordCalibration", "recordCalibration(Lorg/vocaltract/pixel/SingerCalibrationReport;)V", 0);
    }

    /* JADX DEBUG: Method arguments types fixed to match base method, original types: [java.lang.Object] */
    /* JADX DEBUG: Return type fixed from 'java.lang.Object' to match base method */
    @Override // kotlin.jvm.functions.Function1
    public /* bridge */ /* synthetic */ Unit invoke(SingerCalibrationReport singerCalibrationReport) {
        invoke2(singerCalibrationReport);
        return Unit.INSTANCE;
    }

    /* renamed from: invoke, reason: avoid collision after fix types in other method */
    public final void invoke2(SingerCalibrationReport p0) {
        Intrinsics.checkNotNullParameter(p0, "p0");
        ((DiagnosticsRuntime) this.receiver).recordCalibration(p0);
    }
}
