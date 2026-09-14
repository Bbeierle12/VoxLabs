package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.ResultKt;
import kotlin.Unit;
import kotlin.collections.ArraysKt;
import kotlin.coroutines.Continuation;
import kotlin.coroutines.intrinsics.IntrinsicsKt;
import kotlin.coroutines.jvm.internal.DebugMetadata;
import kotlin.coroutines.jvm.internal.RestrictedSuspendLambda;
import kotlin.jvm.functions.Function2;
import kotlin.ranges.IntProgression;
import kotlin.ranges.RangesKt;
import kotlin.sequences.SequenceScope;

/* compiled from: AnatomyBenchActivity.kt */
@Metadata(d1 = {"\u0000\u000e\n\u0000\n\u0002\u0010\u0002\n\u0002\u0018\u0002\n\u0002\u0018\u0002\u0010\u0000\u001a\u00020\u0001*\b\u0012\u0004\u0012\u00020\u00030\u0002H\n"}, d2 = {"<anonymous>", "", "Lkotlin/sequences/SequenceScope;", "Lorg/vocaltract/pixel/PcmFrame;"}, k = 3, mv = {2, 0, 0}, xi = 48)
@DebugMetadata(c = "org.vocaltract.pixel.SharedReplay$frames$1", f = "AnatomyBenchActivity.kt", i = {0, 0, 0}, l = {284}, m = "invokeSuspend", n = {"$this$sequence", "pcm", "start"}, s = {"L$0", "L$1", "I$0"})
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
final class SharedReplay$frames$1 extends RestrictedSuspendLambda implements Function2<SequenceScope<? super PcmFrame>, Continuation<? super Unit>, Object> {
    final /* synthetic */ int $hop;
    final /* synthetic */ int $rate;
    final /* synthetic */ int $size;
    final /* synthetic */ AdditiveSynthState $sound;
    int I$0;
    int I$1;
    int I$2;
    private /* synthetic */ Object L$0;
    Object L$1;
    int label;

    /* JADX WARN: 'super' call moved to the top of the method (can break code semantics) */
    SharedReplay$frames$1(int i, int i2, AdditiveSynthState additiveSynthState, int i3, Continuation<? super SharedReplay$frames$1> continuation) {
        super(2, continuation);
        this.$rate = i;
        this.$size = i2;
        this.$sound = additiveSynthState;
        this.$hop = i3;
    }

    @Override // kotlin.coroutines.jvm.internal.BaseContinuationImpl
    public final Continuation<Unit> create(Object obj, Continuation<?> continuation) {
        SharedReplay$frames$1 sharedReplay$frames$1 = new SharedReplay$frames$1(this.$rate, this.$size, this.$sound, this.$hop, continuation);
        sharedReplay$frames$1.L$0 = obj;
        return sharedReplay$frames$1;
    }

    /* JADX DEBUG: Method merged with bridge method: invoke(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object; */
    @Override // kotlin.jvm.functions.Function2
    public final Object invoke(SequenceScope<? super PcmFrame> sequenceScope, Continuation<? super Unit> continuation) {
        return ((SharedReplay$frames$1) create(sequenceScope, continuation)).invokeSuspend(Unit.INSTANCE);
    }

    /* JADX WARN: Removed duplicated region for block: B:6:0x00ab  */
    /* JADX WARN: Removed duplicated region for block: B:9:0x00a8 A[RETURN] */
    /* JADX WARN: Unsupported multi-entry loop pattern (BACK_EDGE: B:8:0x00a6 -> B:5:0x00a9). Please report as a decompilation issue!!! */
    @Override // kotlin.coroutines.jvm.internal.BaseContinuationImpl
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public final Object invokeSuspend(Object obj) {
        int first;
        SequenceScope sequenceScope;
        short[] sArr;
        int i;
        int i2;
        Object coroutine_suspended = IntrinsicsKt.getCOROUTINE_SUSPENDED();
        int i3 = this.label;
        if (i3 != 0) {
            if (i3 != 1) {
                throw new IllegalStateException("call to 'resume' before 'invoke' with coroutine");
            }
            i = this.I$2;
            i2 = this.I$1;
            first = this.I$0;
            sArr = (short[]) this.L$1;
            sequenceScope = (SequenceScope) this.L$0;
            ResultKt.throwOnFailure(obj);
            if (first != i2) {
                first += i;
                this.L$0 = sequenceScope;
                this.L$1 = sArr;
                this.I$0 = first;
                this.I$1 = i2;
                this.I$2 = i;
                this.label = 1;
                if (sequenceScope.yield(new PcmFrame((first * 1000000000) / this.$rate, ArraysKt.copyOfRange(sArr, first, this.$size + first), "shared-model-replay"), this) == coroutine_suspended) {
                    return coroutine_suspended;
                }
                if (first != i2) {
                }
            }
            return Unit.INSTANCE;
        }
        ResultKt.throwOnFailure(obj);
        SequenceScope sequenceScope2 = (SequenceScope) this.L$0;
        AdditiveSignalGenerator additiveSignalGenerator = new AdditiveSignalGenerator(this.$rate, 0.0f, 0.0f, 6, null);
        int i4 = (this.$rate * 10) + this.$size;
        float[] fArr = new float[i4];
        additiveSignalGenerator.render(this.$sound.withPlaying(true), fArr);
        short[] sArr2 = new short[i4];
        AdditiveSynthesisMath.INSTANCE.floatToPcm16(fArr, sArr2);
        IntProgression step = RangesKt.step(RangesKt.until(0, this.$rate * 10), this.$hop);
        first = step.getFirst();
        int last = step.getLast();
        int step2 = step.getStep();
        if ((step2 > 0 && first <= last) || (step2 < 0 && last <= first)) {
            sequenceScope = sequenceScope2;
            sArr = sArr2;
            i = step2;
            i2 = last;
            this.L$0 = sequenceScope;
            this.L$1 = sArr;
            this.I$0 = first;
            this.I$1 = i2;
            this.I$2 = i;
            this.label = 1;
            if (sequenceScope.yield(new PcmFrame((first * 1000000000) / this.$rate, ArraysKt.copyOfRange(sArr, first, this.$size + first), "shared-model-replay"), this) == coroutine_suspended) {
            }
            if (first != i2) {
            }
        }
        return Unit.INSTANCE;
    }
}
