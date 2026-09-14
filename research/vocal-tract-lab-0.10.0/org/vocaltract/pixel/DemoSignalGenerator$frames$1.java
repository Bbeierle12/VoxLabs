package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.ResultKt;
import kotlin.Unit;
import kotlin.coroutines.Continuation;
import kotlin.coroutines.intrinsics.IntrinsicsKt;
import kotlin.coroutines.jvm.internal.DebugMetadata;
import kotlin.coroutines.jvm.internal.RestrictedSuspendLambda;
import kotlin.jvm.functions.Function2;
import kotlin.ranges.RangesKt;
import kotlin.sequences.SequenceScope;

/* compiled from: ReplaySources.kt */
@Metadata(d1 = {"\u0000\u000e\n\u0000\n\u0002\u0010\u0002\n\u0002\u0018\u0002\n\u0002\u0018\u0002\u0010\u0000\u001a\u00020\u0001*\b\u0012\u0004\u0012\u00020\u00030\u0002H\n"}, d2 = {"<anonymous>", "", "Lkotlin/sequences/SequenceScope;", "Lorg/vocaltract/pixel/PcmFrame;"}, k = 3, mv = {2, 0, 0}, xi = 48)
@DebugMetadata(c = "org.vocaltract.pixel.DemoSignalGenerator$frames$1", f = "ReplaySources.kt", i = {0, 0, 0}, l = {39}, m = "invokeSuspend", n = {"$this$sequence", "frameCount", "frameIndex"}, s = {"L$0", "I$0", "I$1"})
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
final class DemoSignalGenerator$frames$1 extends RestrictedSuspendLambda implements Function2<SequenceScope<? super PcmFrame>, Continuation<? super Unit>, Object> {
    final /* synthetic */ float $durationSeconds;
    final /* synthetic */ float $f0Hz;
    final /* synthetic */ int $frameSize;
    final /* synthetic */ int $sampleRateHz;
    int I$0;
    int I$1;
    private /* synthetic */ Object L$0;
    int label;

    /* JADX WARN: 'super' call moved to the top of the method (can break code semantics) */
    DemoSignalGenerator$frames$1(int i, int i2, float f, float f2, Continuation<? super DemoSignalGenerator$frames$1> continuation) {
        super(2, continuation);
        this.$sampleRateHz = i;
        this.$frameSize = i2;
        this.$f0Hz = f;
        this.$durationSeconds = f2;
    }

    @Override // kotlin.coroutines.jvm.internal.BaseContinuationImpl
    public final Continuation<Unit> create(Object obj, Continuation<?> continuation) {
        DemoSignalGenerator$frames$1 demoSignalGenerator$frames$1 = new DemoSignalGenerator$frames$1(this.$sampleRateHz, this.$frameSize, this.$f0Hz, this.$durationSeconds, continuation);
        demoSignalGenerator$frames$1.L$0 = obj;
        return demoSignalGenerator$frames$1;
    }

    /* JADX DEBUG: Method merged with bridge method: invoke(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object; */
    @Override // kotlin.jvm.functions.Function2
    public final Object invoke(SequenceScope<? super PcmFrame> sequenceScope, Continuation<? super Unit> continuation) {
        return ((DemoSignalGenerator$frames$1) create(sequenceScope, continuation)).invokeSuspend(Unit.INSTANCE);
    }

    /* JADX WARN: Removed duplicated region for block: B:24:0x016f  */
    /* JADX WARN: Removed duplicated region for block: B:7:0x0052  */
    /* JADX WARN: Unsupported multi-entry loop pattern (BACK_EDGE: B:18:0x0160 -> B:5:0x0163). Please report as a decompilation issue!!! */
    @Override // kotlin.coroutines.jvm.internal.BaseContinuationImpl
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public final Object invokeSuspend(Object obj) {
        int i;
        int ceil;
        SequenceScope sequenceScope;
        int i2;
        Object obj2;
        int i3;
        int i4;
        SequenceScope sequenceScope2;
        int i5;
        double sq;
        double sq2;
        DemoSignalGenerator$frames$1 demoSignalGenerator$frames$1 = this;
        Object coroutine_suspended = IntrinsicsKt.getCOROUTINE_SUSPENDED();
        int i6 = demoSignalGenerator$frames$1.label;
        int i7 = 1;
        if (i6 == 0) {
            ResultKt.throwOnFailure(obj);
            SequenceScope sequenceScope3 = (SequenceScope) demoSignalGenerator$frames$1.L$0;
            if (demoSignalGenerator$frames$1.$sampleRateHz > 0 && (i = demoSignalGenerator$frames$1.$frameSize) > 0 && demoSignalGenerator$frames$1.$f0Hz > 0.0f) {
                if (demoSignalGenerator$frames$1.$durationSeconds > 0.0f) {
                    ceil = (int) Math.ceil((r7 * r5) / i);
                    sequenceScope = sequenceScope3;
                    i2 = 0;
                    if (i2 < ceil) {
                    }
                }
            }
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (i6 != 1) {
            throw new IllegalStateException("call to 'resume' before 'invoke' with coroutine");
        }
        int i8 = demoSignalGenerator$frames$1.I$1;
        ceil = demoSignalGenerator$frames$1.I$0;
        SequenceScope sequenceScope4 = (SequenceScope) demoSignalGenerator$frames$1.L$0;
        ResultKt.throwOnFailure(obj);
        SequenceScope sequenceScope5 = sequenceScope4;
        int i9 = 1;
        Object obj3 = coroutine_suspended;
        int i10 = i8;
        DemoSignalGenerator$frames$1 demoSignalGenerator$frames$12 = demoSignalGenerator$frames$1;
        int i11 = i10 + 1;
        coroutine_suspended = obj3;
        i7 = i9;
        sequenceScope = sequenceScope5;
        DemoSignalGenerator$frames$1 demoSignalGenerator$frames$13 = demoSignalGenerator$frames$12;
        i2 = i11;
        demoSignalGenerator$frames$1 = demoSignalGenerator$frames$13;
        if (i2 < ceil) {
            int i12 = demoSignalGenerator$frames$1.$frameSize;
            short[] sArr = new short[i12];
            int i13 = 0;
            while (i13 < i12) {
                double d = ((i2 * demoSignalGenerator$frames$1.$frameSize) + i13) / demoSignalGenerator$frames$1.$sampleRateHz;
                int coerceAtMost = RangesKt.coerceAtMost((int) Math.floor((r12 / 2.0f) / demoSignalGenerator$frames$1.$f0Hz), 32);
                double d2 = 0.0d;
                if (i7 <= coerceAtMost) {
                    int i14 = i7;
                    while (true) {
                        i4 = ceil;
                        double d3 = i14;
                        i5 = i12;
                        obj2 = coroutine_suspended;
                        double d4 = i14 * demoSignalGenerator$frames$1.$f0Hz;
                        i3 = i2;
                        sq = DemoSignalGenerator.INSTANCE.sq((d4 - 750.0d) / 180.0d);
                        double exp = (1.0d / d3) * ((Math.exp(sq * (-0.5d)) * 2.0d) + 1.0d);
                        sequenceScope2 = sequenceScope;
                        sq2 = DemoSignalGenerator.INSTANCE.sq((d4 - 1250.0d) / 260.0d);
                        d2 += exp * ((Math.exp(sq2 * (-0.5d)) * 1.4d) + 1.0d) * Math.sin((d4 * 6.283185307179586d * d) + (d3 * 0.17d));
                        if (i14 != coerceAtMost) {
                            i14++;
                            demoSignalGenerator$frames$1 = this;
                            ceil = i4;
                            sequenceScope = sequenceScope2;
                            i12 = i5;
                            coroutine_suspended = obj2;
                            i2 = i3;
                        }
                    }
                } else {
                    obj2 = coroutine_suspended;
                    i3 = i2;
                    i4 = ceil;
                    sequenceScope2 = sequenceScope;
                    i5 = i12;
                }
                sArr[i13] = (short) RangesKt.coerceIn((int) (d2 * 2100.0d), -32768, 32767);
                i13++;
                demoSignalGenerator$frames$1 = this;
                ceil = i4;
                sequenceScope = sequenceScope2;
                i12 = i5;
                coroutine_suspended = obj2;
                i2 = i3;
                i7 = 1;
            }
            Object obj4 = coroutine_suspended;
            sequenceScope5 = sequenceScope;
            i10 = i2;
            demoSignalGenerator$frames$12 = this;
            demoSignalGenerator$frames$12.L$0 = sequenceScope5;
            ceil = ceil;
            demoSignalGenerator$frames$12.I$0 = ceil;
            demoSignalGenerator$frames$12.I$1 = i10;
            i9 = 1;
            demoSignalGenerator$frames$12.label = 1;
            obj3 = obj4;
            if (sequenceScope5.yield(new PcmFrame(((i10 * demoSignalGenerator$frames$12.$frameSize) * 1000000000) / demoSignalGenerator$frames$12.$sampleRateHz, sArr, "offline-demo"), demoSignalGenerator$frames$12) == obj3) {
                return obj3;
            }
            int i112 = i10 + 1;
            coroutine_suspended = obj3;
            i7 = i9;
            sequenceScope = sequenceScope5;
            DemoSignalGenerator$frames$1 demoSignalGenerator$frames$132 = demoSignalGenerator$frames$12;
            i2 = i112;
            demoSignalGenerator$frames$1 = demoSignalGenerator$frames$132;
            if (i2 < ceil) {
                return Unit.INSTANCE;
            }
        }
    }
}
