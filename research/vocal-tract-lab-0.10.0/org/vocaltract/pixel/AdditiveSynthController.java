package org.vocaltract.pixel;

import java.util.concurrent.atomic.AtomicReference;
import kotlin.Metadata;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import org.vocaltract.pixel.AdditiveSynthState;

/* compiled from: AdditiveSynthesis.kt */
@Metadata(d1 = {"\u00004\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\b\u0005\n\u0002\u0010\u0007\n\u0002\b\u0003\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0018\u0002\u0018\u00002\u00020\u0001B\u0011\u0012\b\b\u0002\u0010\u0002\u001a\u00020\u0003¢\u0006\u0004\b\u0004\u0010\u0005J\u0006\u0010\t\u001a\u00020\u0003J\u000e\u0010\n\u001a\u00020\u00032\u0006\u0010\u000b\u001a\u00020\u0003J\u000e\u0010\f\u001a\u00020\u00032\u0006\u0010\u000b\u001a\u00020\rJ\u000e\u0010\u000e\u001a\u00020\u00032\u0006\u0010\u000b\u001a\u00020\rJ\u0016\u0010\u000f\u001a\u00020\u00032\u0006\u0010\u0010\u001a\u00020\u00112\u0006\u0010\u000b\u001a\u00020\rJ\u000e\u0010\u0012\u001a\u00020\u00032\u0006\u0010\u000b\u001a\u00020\u0013J\u001c\u0010\u0014\u001a\u00020\u00032\u0012\u0010\u0015\u001a\u000e\u0012\u0004\u0012\u00020\u0003\u0012\u0004\u0012\u00020\u00030\u0016H\u0002R\u001c\u0010\u0006\u001a\u0010\u0012\f\u0012\n \b*\u0004\u0018\u00010\u00030\u00030\u0007X\u0082\u0004¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/AdditiveSynthController;", "", "initial", "Lorg/vocaltract/pixel/AdditiveSynthState;", "<init>", "(Lorg/vocaltract/pixel/AdditiveSynthState;)V", "state", "Ljava/util/concurrent/atomic/AtomicReference;", "kotlin.jvm.PlatformType", "snapshot", "replace", "value", "setFundamentalHz", "", "setMasterGain", "setPartialGain", "index", "", "setPlaying", "", "update", "transform", "Lkotlin/Function1;"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AdditiveSynthController {
    private final AtomicReference<AdditiveSynthState> state;

    /* JADX DEBUG: Multi-variable search result rejected for r0v1, resolved type: java.lang.Object[] */
    /* JADX WARN: Multi-variable type inference failed */
    public AdditiveSynthController() {
        this(null, 1, 0 == true ? 1 : 0);
    }

    public AdditiveSynthController(AdditiveSynthState initial) {
        Intrinsics.checkNotNullParameter(initial, "initial");
        this.state = new AtomicReference<>(initial);
    }

    public /* synthetic */ AdditiveSynthController(AdditiveSynthState additiveSynthState, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this((i & 1) != 0 ? AdditiveSynthState.Companion.create$default(AdditiveSynthState.INSTANCE, 0.0f, 0.0f, null, false, 15, null) : additiveSynthState);
    }

    public final AdditiveSynthState snapshot() {
        AdditiveSynthState additiveSynthState = this.state.get();
        Intrinsics.checkNotNullExpressionValue(additiveSynthState, "get(...)");
        return additiveSynthState;
    }

    public final AdditiveSynthState replace(AdditiveSynthState value) {
        Intrinsics.checkNotNullParameter(value, "value");
        this.state.set(value);
        return value;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final AdditiveSynthState setFundamentalHz$lambda$0(float f, AdditiveSynthState it) {
        Intrinsics.checkNotNullParameter(it, "it");
        return it.withFundamentalHz(f);
    }

    public final AdditiveSynthState setFundamentalHz(final float value) {
        return update(new Function1() { // from class: org.vocaltract.pixel.AdditiveSynthController$$ExternalSyntheticLambda3
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                AdditiveSynthState fundamentalHz$lambda$0;
                fundamentalHz$lambda$0 = AdditiveSynthController.setFundamentalHz$lambda$0(value, (AdditiveSynthState) obj);
                return fundamentalHz$lambda$0;
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final AdditiveSynthState setMasterGain$lambda$1(float f, AdditiveSynthState it) {
        Intrinsics.checkNotNullParameter(it, "it");
        return it.withMasterGain(f);
    }

    public final AdditiveSynthState setMasterGain(final float value) {
        return update(new Function1() { // from class: org.vocaltract.pixel.AdditiveSynthController$$ExternalSyntheticLambda1
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                AdditiveSynthState masterGain$lambda$1;
                masterGain$lambda$1 = AdditiveSynthController.setMasterGain$lambda$1(value, (AdditiveSynthState) obj);
                return masterGain$lambda$1;
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final AdditiveSynthState setPartialGain$lambda$2(int i, float f, AdditiveSynthState it) {
        Intrinsics.checkNotNullParameter(it, "it");
        return it.withPartialGain(i, f);
    }

    public final AdditiveSynthState setPartialGain(final int index, final float value) {
        return update(new Function1() { // from class: org.vocaltract.pixel.AdditiveSynthController$$ExternalSyntheticLambda2
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                AdditiveSynthState partialGain$lambda$2;
                partialGain$lambda$2 = AdditiveSynthController.setPartialGain$lambda$2(index, value, (AdditiveSynthState) obj);
                return partialGain$lambda$2;
            }
        });
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final AdditiveSynthState setPlaying$lambda$3(boolean z, AdditiveSynthState it) {
        Intrinsics.checkNotNullParameter(it, "it");
        return it.withPlaying(z);
    }

    public final AdditiveSynthState setPlaying(final boolean value) {
        return update(new Function1() { // from class: org.vocaltract.pixel.AdditiveSynthController$$ExternalSyntheticLambda4
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                AdditiveSynthState playing$lambda$3;
                playing$lambda$3 = AdditiveSynthController.setPlaying$lambda$3(value, (AdditiveSynthState) obj);
                return playing$lambda$3;
            }
        });
    }

    private final AdditiveSynthState update(Function1<? super AdditiveSynthState, AdditiveSynthState> transform) {
        AdditiveSynthState additiveSynthState;
        AdditiveSynthState invoke;
        do {
            additiveSynthState = this.state.get();
            Intrinsics.checkNotNull(additiveSynthState);
            invoke = transform.invoke(additiveSynthState);
        } while (!AdditiveSynthController$$ExternalSyntheticBackportWithForwarding0.m(this.state, additiveSynthState, invoke));
        return invoke;
    }
}
