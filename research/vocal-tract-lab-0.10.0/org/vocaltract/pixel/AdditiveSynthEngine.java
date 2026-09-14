package org.vocaltract.pixel;

import android.media.AudioAttributes;
import android.media.AudioFormat;
import android.media.AudioTrack;
import android.os.Process;
import java.util.Arrays;
import java.util.concurrent.atomic.AtomicBoolean;
import kotlin.Metadata;
import kotlin.Unit;
import kotlin.concurrent.ThreadsKt;
import kotlin.io.ConstantsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: AdditiveSynthEngine.kt */
@Metadata(d1 = {"\u0000d\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\u0010\u0014\n\u0002\u0010\u0002\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0005\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010\u0007\n\u0002\b\u000b\n\u0002\u0010\u0017\n\u0002\b\u0005\u0018\u0000 32\u00020\u0001:\u00013BS\u0012\b\b\u0002\u0010\u0002\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0004\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0005\u001a\u00020\u0006\u0012\u0016\b\u0002\u0010\u0007\u001a\u0010\u0012\u0004\u0012\u00020\t\u0012\u0004\u0012\u00020\n\u0018\u00010\b\u0012\u0014\b\u0002\u0010\u000b\u001a\u000e\u0012\u0004\u0012\u00020\f\u0012\u0004\u0012\u00020\n0\b¢\u0006\u0004\b\r\u0010\u000eJ\u0006\u0010\u001e\u001a\u00020\u001fJ\u000e\u0010 \u001a\u00020\u001f2\u0006\u0010!\u001a\u00020\"J\u000e\u0010#\u001a\u00020\u001f2\u0006\u0010!\u001a\u00020\"J\u0016\u0010$\u001a\u00020\u001f2\u0006\u0010%\u001a\u00020\u00032\u0006\u0010!\u001a\u00020\"J\u0006\u0010&\u001a\u00020\u001cJ\u0006\u0010'\u001a\u00020\nJ\u0006\u0010(\u001a\u00020\nJ\n\u0010)\u001a\u0004\u0018\u00010\u0017H\u0002J\u0010\u0010*\u001a\u00020\n2\u0006\u0010+\u001a\u00020\u0017H\u0002J\u0018\u0010,\u001a\u00020\u001c2\u0006\u0010+\u001a\u00020\u00172\u0006\u0010-\u001a\u00020.H\u0002J\u0010\u0010/\u001a\u00020\n2\u0006\u0010-\u001a\u00020\tH\u0002J\u0010\u00100\u001a\u00020\n2\u0006\u00101\u001a\u00020\fH\u0002J\u0012\u00102\u001a\u00020\n2\b\u0010+\u001a\u0004\u0018\u00010\u0017H\u0002R\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0004\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u0011\u0010\u0005\u001a\u00020\u0006¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u0010R\u001c\u0010\u0007\u001a\u0010\u0012\u0004\u0012\u00020\t\u0012\u0004\u0012\u00020\n\u0018\u00010\bX\u0082\u0004¢\u0006\u0002\n\u0000R\u001a\u0010\u000b\u001a\u000e\u0012\u0004\u0012\u00020\f\u0012\u0004\u0012\u00020\n0\bX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0011\u001a\u00020\u0012X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0013\u001a\u00020\u0012X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0014\u001a\u00020\u0012X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0015\u001a\u00020\u0001X\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\u0016\u001a\u0004\u0018\u00010\u0017X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\u0018\u001a\u0004\u0018\u00010\u0019X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\u001a\u001a\u0004\u0018\u00010\u0017X\u0082\u000e¢\u0006\u0002\n\u0000R\u0011\u0010\u001b\u001a\u00020\u001c8F¢\u0006\u0006\u001a\u0004\b\u001b\u0010\u001d"}, d2 = {"Lorg/vocaltract/pixel/AdditiveSynthEngine;", "", "sampleRateHz", "", "framesPerBuffer", "controller", "Lorg/vocaltract/pixel/AdditiveSynthController;", "onWaveform", "Lkotlin/Function1;", "", "", "onError", "", "<init>", "(IILorg/vocaltract/pixel/AdditiveSynthController;Lkotlin/jvm/functions/Function1;Lkotlin/jvm/functions/Function1;)V", "getController", "()Lorg/vocaltract/pixel/AdditiveSynthController;", "running", "Ljava/util/concurrent/atomic/AtomicBoolean;", "stopRequested", "released", "trackShutdownLock", "audioTrack", "Landroid/media/AudioTrack;", "worker", "Ljava/lang/Thread;", "lastReleasedTrack", "isRunning", "", "()Z", "state", "Lorg/vocaltract/pixel/AdditiveSynthState;", "setFundamentalHz", "value", "", "setMasterGain", "setPartialGain", "index", "start", "stop", "release", "createAudioTrack", "runAudioLoop", "track", "writeFully", "buffer", "", "publishWaveform", "reportError", "message", "shutdownTrack", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AdditiveSynthEngine {

    @Deprecated
    public static final int BUFFER_QUEUE_DEPTH = 4;
    private static final Companion Companion = new Companion(null);

    @Deprecated
    public static final long FORCED_JOIN_TIMEOUT_MS = 250;

    @Deprecated
    public static final int STOP_FADE_BUFFERS = 4;

    @Deprecated
    public static final long STOP_JOIN_TIMEOUT_MS = 600;

    @Deprecated
    public static final int WAVEFORM_CALLBACK_INTERVAL_BUFFERS = 4;
    private volatile AudioTrack audioTrack;
    private final AdditiveSynthController controller;
    private final int framesPerBuffer;
    private volatile AudioTrack lastReleasedTrack;
    private final Function1<String, Unit> onError;
    private final Function1<float[], Unit> onWaveform;
    private final AtomicBoolean released;
    private final AtomicBoolean running;
    private final int sampleRateHz;
    private final AtomicBoolean stopRequested;
    private final Object trackShutdownLock;
    private volatile Thread worker;

    public AdditiveSynthEngine() {
        this(0, 0, null, null, null, 31, null);
    }

    /* JADX DEBUG: Multi-variable search result rejected for r5v0, resolved type: kotlin.jvm.functions.Function1<? super float[], kotlin.Unit> */
    /* JADX DEBUG: Multi-variable search result rejected for r6v0, resolved type: kotlin.jvm.functions.Function1<? super java.lang.String, kotlin.Unit> */
    /* JADX WARN: Multi-variable type inference failed */
    public AdditiveSynthEngine(int i, int i2, AdditiveSynthController controller, Function1<? super float[], Unit> function1, Function1<? super String, Unit> onError) {
        Intrinsics.checkNotNullParameter(controller, "controller");
        Intrinsics.checkNotNullParameter(onError, "onError");
        this.sampleRateHz = i;
        this.framesPerBuffer = i2;
        this.controller = controller;
        this.onWaveform = function1;
        this.onError = onError;
        this.running = new AtomicBoolean(false);
        this.stopRequested = new AtomicBoolean(false);
        this.released = new AtomicBoolean(false);
        this.trackShutdownLock = new Object();
        if (8000 > i || i >= 192001) {
            throw new IllegalArgumentException(("Unsupported synthesis sample rate: " + i).toString());
        }
        if (128 > i2 || i2 >= 8193) {
            throw new IllegalArgumentException("framesPerBuffer must be in 128..8192".toString());
        }
    }

    public /* synthetic */ AdditiveSynthEngine(int i, int i2, AdditiveSynthController additiveSynthController, Function1 function1, Function1 function12, int i3, DefaultConstructorMarker defaultConstructorMarker) {
        this((i3 & 1) != 0 ? SummedWaveformView.DISPLAY_SAMPLE_RATE_HZ : i, (i3 & 2) != 0 ? ConstantsKt.MINIMUM_BLOCK_SIZE : i2, (i3 & 4) != 0 ? new AdditiveSynthController(null, 1, null) : additiveSynthController, (i3 & 8) == 0 ? function1 : null, (i3 & 16) != 0 ? new Function1() { // from class: org.vocaltract.pixel.AdditiveSynthEngine$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                Unit _init_$lambda$0;
                _init_$lambda$0 = AdditiveSynthEngine._init_$lambda$0((String) obj);
                return _init_$lambda$0;
            }
        } : function12);
    }

    public final AdditiveSynthController getController() {
        return this.controller;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit _init_$lambda$0(String it) {
        Intrinsics.checkNotNullParameter(it, "it");
        return Unit.INSTANCE;
    }

    public final boolean isRunning() {
        return this.running.get();
    }

    public final AdditiveSynthState state() {
        return this.controller.snapshot();
    }

    public final AdditiveSynthState setFundamentalHz(float value) {
        return this.controller.setFundamentalHz(value);
    }

    public final AdditiveSynthState setMasterGain(float value) {
        return this.controller.setMasterGain(value);
    }

    public final AdditiveSynthState setPartialGain(int index, float value) {
        return this.controller.setPartialGain(index, value);
    }

    public final synchronized boolean start() {
        if (this.running.get()) {
            return true;
        }
        Thread thread = this.worker;
        if (thread != null && thread.isAlive()) {
            reportError("The previous synthesizer worker is still shutting down");
            return false;
        }
        if (this.released.get()) {
            reportError("Synthesizer has already been released");
            return false;
        }
        final AudioTrack createAudioTrack = createAudioTrack();
        if (createAudioTrack == null) {
            return false;
        }
        try {
            createAudioTrack.play();
            this.stopRequested.set(false);
            this.controller.setPlaying(true);
            this.audioTrack = createAudioTrack;
            this.running.set(true);
            this.worker = ThreadsKt.thread$default(false, true, null, "additive-synth", 0, new Function0() { // from class: org.vocaltract.pixel.AdditiveSynthEngine$$ExternalSyntheticLambda1
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function0
                public final Object invoke() {
                    Unit start$lambda$3;
                    start$lambda$3 = AdditiveSynthEngine.start$lambda$3(AdditiveSynthEngine.this, createAudioTrack);
                    return start$lambda$3;
                }
            }, 21, null);
            return true;
        } catch (IllegalStateException e) {
            createAudioTrack.release();
            reportError("Synthesizer playback could not start: " + e.getMessage());
            return false;
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit start$lambda$3(AdditiveSynthEngine additiveSynthEngine, AudioTrack audioTrack) {
        additiveSynthEngine.runAudioLoop(audioTrack);
        return Unit.INSTANCE;
    }

    public final synchronized void stop() {
        this.controller.setPlaying(false);
        if (this.running.get() || this.audioTrack != null) {
            this.stopRequested.set(true);
            Thread thread = this.worker;
            if (thread != null) {
                try {
                    thread.join(600L);
                } catch (InterruptedException unused) {
                    Thread.currentThread().interrupt();
                }
            }
            if (thread != null && thread.isAlive()) {
                shutdownTrack(this.audioTrack);
                try {
                    thread.join(250L);
                } catch (InterruptedException unused2) {
                    Thread.currentThread().interrupt();
                }
            }
            shutdownTrack(this.audioTrack);
            this.audioTrack = null;
            this.worker = null;
            this.running.set(false);
        }
    }

    public final void release() {
        if (this.released.compareAndSet(false, true)) {
            stop();
        }
    }

    private final AudioTrack createAudioTrack() {
        int minBufferSize = AudioTrack.getMinBufferSize(this.sampleRateHz, 4, 2);
        if (minBufferSize <= 0) {
            reportError("AudioTrack does not support " + this.sampleRateHz + " Hz mono PCM");
            return null;
        }
        try {
            AudioTrack build = new AudioTrack.Builder().setAudioAttributes(new AudioAttributes.Builder().setUsage(1).setContentType(2).build()).setAudioFormat(new AudioFormat.Builder().setEncoding(2).setSampleRate(this.sampleRateHz).setChannelMask(4).build()).setTransferMode(1).setPerformanceMode(1).setBufferSizeInBytes(Math.max(minBufferSize, this.framesPerBuffer * 8)).build();
            Intrinsics.checkNotNull(build);
            if (build.getState() == 1) {
                return build;
            }
            build.release();
            reportError("AudioTrack synthesizer initialization failed");
            return null;
        } catch (RuntimeException e) {
            reportError("Could not initialize synthesizer output: " + e.getMessage());
            return null;
        }
    }

    /* JADX DEBUG: Another duplicated slice has different insns count: {[INVOKE, IGET]}, finally: {[INVOKE, IGET, IPUT, IGET, INVOKE, IF, IGET, INVOKE, IPUT, IGET, INVOKE, IF] complete} */
    private final void runAudioLoop(AudioTrack track) {
        Process.setThreadPriority(-16);
        AdditiveSignalGenerator additiveSignalGenerator = new AdditiveSignalGenerator(this.sampleRateHz, 0.0f, 0.0f, 6, null);
        int i = this.framesPerBuffer;
        float[] fArr = new float[i];
        short[] sArr = new short[i];
        int i2 = 0;
        while (!this.stopRequested.get()) {
            try {
                try {
                    additiveSignalGenerator.render(this.controller.snapshot(), fArr);
                    AdditiveSynthesisMath.INSTANCE.floatToPcm16(fArr, sArr);
                    if (!writeFully(track, sArr)) {
                        break;
                    }
                    if (i2 == 0) {
                        publishWaveform(fArr);
                        i2 = 4;
                    }
                    i2--;
                } catch (RuntimeException e) {
                    if (!this.stopRequested.get()) {
                        reportError("Synthesizer worker failed: " + e.getMessage());
                    }
                    shutdownTrack(track);
                    if (this.audioTrack == track) {
                        this.audioTrack = null;
                    }
                    if (this.worker != Thread.currentThread()) {
                        return;
                    }
                }
            } catch (Throwable th) {
                shutdownTrack(track);
                if (this.audioTrack == track) {
                    this.audioTrack = null;
                }
                if (this.worker == Thread.currentThread()) {
                    this.worker = null;
                    this.running.set(false);
                }
                throw th;
            }
        }
        if (this.stopRequested.get()) {
            AdditiveSynthState withPlaying = this.controller.snapshot().withPlaying(false);
            for (int i3 = 0; i3 < 4; i3++) {
                additiveSignalGenerator.render(withPlaying, fArr);
                AdditiveSynthesisMath.INSTANCE.floatToPcm16(fArr, sArr);
                if (!writeFully(track, sArr)) {
                    break;
                }
            }
            publishWaveform(new float[this.framesPerBuffer]);
        }
        shutdownTrack(track);
        if (this.audioTrack == track) {
            this.audioTrack = null;
        }
        if (this.worker != Thread.currentThread()) {
            return;
        }
        this.worker = null;
        this.running.set(false);
    }

    private final boolean writeFully(AudioTrack track, short[] buffer) {
        int i = 0;
        while (i < buffer.length) {
            int write = track.write(buffer, i, buffer.length - i, 0);
            if (write < 0) {
                if (!this.stopRequested.get()) {
                    reportError("AudioTrack write failed with code " + write);
                }
                return false;
            }
            if (write == 0 && this.stopRequested.get()) {
                return false;
            }
            i += write;
        }
        return true;
    }

    private final void publishWaveform(float[] buffer) {
        try {
            Function1<float[], Unit> function1 = this.onWaveform;
            if (function1 != null) {
                float[] copyOf = Arrays.copyOf(buffer, buffer.length);
                Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
                function1.invoke(copyOf);
            }
        } catch (RuntimeException unused) {
        }
    }

    private final void reportError(String message) {
        try {
            this.onError.invoke(message);
        } catch (RuntimeException unused) {
        }
    }

    private final void shutdownTrack(AudioTrack track) {
        if (track == null) {
            return;
        }
        synchronized (this.trackShutdownLock) {
            if (this.lastReleasedTrack == track) {
                return;
            }
            this.lastReleasedTrack = track;
            try {
                if (track.getPlayState() == 3) {
                    track.stop();
                }
            } catch (IllegalStateException unused) {
            }
            try {
                track.flush();
            } catch (IllegalStateException unused2) {
            }
            track.release();
            Unit unit = Unit.INSTANCE;
        }
    }

    /* compiled from: AdditiveSynthEngine.kt */
    @Metadata(d1 = {"\u0000\u001a\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\b\n\u0002\b\u0003\n\u0002\u0010\t\n\u0000\b\u0082\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003R\u000e\u0010\u0004\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\u0007\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\b\u001a\u00020\tX\u0086T¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\tX\u0086T¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/AdditiveSynthEngine$Companion;", "", "<init>", "()V", "BUFFER_QUEUE_DEPTH", "", "WAVEFORM_CALLBACK_INTERVAL_BUFFERS", "STOP_FADE_BUFFERS", "STOP_JOIN_TIMEOUT_MS", "", "FORCED_JOIN_TIMEOUT_MS"}, k = 1, mv = {2, 0, 0}, xi = 48)
    private static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }
    }
}
