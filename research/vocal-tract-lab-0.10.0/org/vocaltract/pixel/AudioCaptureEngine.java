package org.vocaltract.pixel;

import android.content.Context;
import android.media.AudioFormat;
import android.media.AudioManager;
import android.media.AudioRecord;
import android.os.Process;
import java.util.Arrays;
import java.util.concurrent.atomic.AtomicBoolean;
import kotlin.Metadata;
import kotlin.Unit;
import kotlin.concurrent.ThreadsKt;
import kotlin.jvm.functions.Function0;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.text.StringsKt;

/* compiled from: AudioCaptureEngine.kt */
@Metadata(d1 = {"\u0000H\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\b\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0002\u0010\u0002\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0003\u0018\u00002\u00020\u0001BQ\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0007\u001a\u00020\u0005\u0012\u0012\u0010\b\u001a\u000e\u0012\u0004\u0012\u00020\n\u0012\u0004\u0012\u00020\u000b0\t\u0012\u0012\u0010\f\u001a\u000e\u0012\u0004\u0012\u00020\r\u0012\u0004\u0012\u00020\u000b0\t¢\u0006\u0004\b\u000e\u0010\u000fJ\b\u0010\u0019\u001a\u00020\u0017H\u0007J\u0006\u0010\u001a\u001a\u00020\u000bR\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0004\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0007\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u001a\u0010\b\u001a\u000e\u0012\u0004\u0012\u00020\n\u0012\u0004\u0012\u00020\u000b0\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u001a\u0010\f\u001a\u000e\u0012\u0004\u0012\u00020\r\u0012\u0004\u0012\u00020\u000b0\tX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0010\u001a\u00020\u0011X\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\u0012\u001a\u0004\u0018\u00010\u0013X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\u0014\u001a\u0004\u0018\u00010\u0015X\u0082\u000e¢\u0006\u0002\n\u0000R\u0011\u0010\u0016\u001a\u00020\u00178F¢\u0006\u0006\u001a\u0004\b\u0016\u0010\u0018"}, d2 = {"Lorg/vocaltract/pixel/AudioCaptureEngine;", "", "context", "Landroid/content/Context;", "sampleRateHz", "", "frameSize", "hopSize", "onFrame", "Lkotlin/Function1;", "Lorg/vocaltract/pixel/PcmFrame;", "", "onError", "", "<init>", "(Landroid/content/Context;IIILkotlin/jvm/functions/Function1;Lkotlin/jvm/functions/Function1;)V", "running", "Ljava/util/concurrent/atomic/AtomicBoolean;", "recorder", "Landroid/media/AudioRecord;", "worker", "Ljava/lang/Thread;", "isRunning", "", "()Z", "start", "stop"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AudioCaptureEngine {
    private final Context context;
    private final int frameSize;
    private final int hopSize;
    private final Function1<String, Unit> onError;
    private final Function1<PcmFrame, Unit> onFrame;
    private volatile AudioRecord recorder;
    private final AtomicBoolean running;
    private final int sampleRateHz;
    private volatile Thread worker;

    /* JADX DEBUG: Multi-variable search result rejected for r6v0, resolved type: kotlin.jvm.functions.Function1<? super org.vocaltract.pixel.PcmFrame, kotlin.Unit> */
    /* JADX DEBUG: Multi-variable search result rejected for r7v0, resolved type: kotlin.jvm.functions.Function1<? super java.lang.String, kotlin.Unit> */
    /* JADX WARN: Multi-variable type inference failed */
    public AudioCaptureEngine(Context context, int i, int i2, int i3, Function1<? super PcmFrame, Unit> onFrame, Function1<? super String, Unit> onError) {
        Intrinsics.checkNotNullParameter(context, "context");
        Intrinsics.checkNotNullParameter(onFrame, "onFrame");
        Intrinsics.checkNotNullParameter(onError, "onError");
        this.context = context;
        this.sampleRateHz = i;
        this.frameSize = i2;
        this.hopSize = i3;
        this.onFrame = onFrame;
        this.onError = onError;
        if (1 > i3 || i3 > i2) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        this.running = new AtomicBoolean(false);
    }

    public /* synthetic */ AudioCaptureEngine(Context context, int i, int i2, int i3, Function1 function1, Function1 function12, int i4, DefaultConstructorMarker defaultConstructorMarker) {
        this(context, i, i2, (i4 & 8) != 0 ? i2 : i3, function1, function12);
    }

    public final boolean isRunning() {
        return this.running.get();
    }

    public final boolean start() {
        String property;
        if (this.running.get()) {
            return true;
        }
        if (this.context.checkSelfPermission("android.permission.RECORD_AUDIO") != 0) {
            this.onError.invoke("Microphone permission is not granted");
            return false;
        }
        int minBufferSize = AudioRecord.getMinBufferSize(this.sampleRateHz, 16, 2);
        if (minBufferSize <= 0) {
            this.onError.invoke("AudioRecord does not support the requested PCM format");
            return false;
        }
        AudioManager audioManager = (AudioManager) this.context.getSystemService(AudioManager.class);
        try {
            final AudioRecord build = new AudioRecord.Builder().setAudioSource((audioManager == null || (property = audioManager.getProperty("android.media.property.SUPPORT_AUDIO_SOURCE_UNPROCESSED")) == null) ? false : Intrinsics.areEqual((Object) StringsKt.toBooleanStrictOrNull(property), (Object) true) ? 9 : 6).setAudioFormat(new AudioFormat.Builder().setSampleRate(this.sampleRateHz).setEncoding(2).setChannelMask(16).build()).setBufferSizeInBytes(Math.max(minBufferSize, this.frameSize * 8)).build();
            if (build.getState() != 1) {
                build.release();
                this.onError.invoke("AudioRecord initialization failed");
                return false;
            }
            this.recorder = build;
            this.running.set(true);
            try {
                build.startRecording();
                this.worker = ThreadsKt.thread$default(false, true, null, "pcm-capture", 0, new Function0() { // from class: org.vocaltract.pixel.AudioCaptureEngine$$ExternalSyntheticLambda0
                    /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                    @Override // kotlin.jvm.functions.Function0
                    public final Object invoke() {
                        Unit start$lambda$0;
                        start$lambda$0 = AudioCaptureEngine.start$lambda$0(AudioCaptureEngine.this, build);
                        return start$lambda$0;
                    }
                }, 21, null);
                return true;
            } catch (IllegalStateException e) {
                this.running.set(false);
                build.release();
                this.recorder = null;
                this.onError.invoke("Microphone start failed: " + e.getMessage());
                return false;
            }
        } catch (RuntimeException e2) {
            this.onError.invoke("Could not initialize microphone: " + e2.getMessage());
            return false;
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final Unit start$lambda$0(AudioCaptureEngine audioCaptureEngine, AudioRecord audioRecord) {
        Process.setThreadPriority(-16);
        int i = audioCaptureEngine.hopSize;
        short[] sArr = new short[i];
        int i2 = audioCaptureEngine.frameSize;
        short[] sArr2 = new short[i2];
        int i3 = 0;
        while (audioCaptureEngine.running.get()) {
            int i4 = 0;
            while (true) {
                if (i4 >= i || !audioCaptureEngine.running.get()) {
                    break;
                }
                int read = audioRecord.read(sArr, i4, i - i4, 0);
                if (read < 0) {
                    if (audioCaptureEngine.running.get()) {
                        audioCaptureEngine.onError.invoke("Microphone read failed with AudioRecord code " + read);
                    }
                    audioCaptureEngine.running.set(false);
                } else {
                    i4 += read;
                }
            }
            if (i4 == i) {
                int i5 = audioCaptureEngine.hopSize;
                int i6 = audioCaptureEngine.frameSize;
                if (i5 < i6) {
                    System.arraycopy(sArr2, i5, sArr2, 0, i6 - i5);
                }
                int i7 = audioCaptureEngine.frameSize;
                int i8 = audioCaptureEngine.hopSize;
                System.arraycopy(sArr, 0, sArr2, i7 - i8, i8);
                i3 = Math.min(audioCaptureEngine.frameSize, i3 + audioCaptureEngine.hopSize);
                if (i3 == audioCaptureEngine.frameSize) {
                    Function1<PcmFrame, Unit> function1 = audioCaptureEngine.onFrame;
                    long nanoTime = System.nanoTime();
                    short[] copyOf = Arrays.copyOf(sArr2, i2);
                    Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
                    function1.invoke(new PcmFrame(nanoTime, copyOf, "live"));
                }
            }
        }
        return Unit.INSTANCE;
    }

    public final void stop() {
        if (this.running.getAndSet(false)) {
            AudioRecord audioRecord = this.recorder;
            if (audioRecord != null) {
                try {
                    audioRecord.stop();
                } catch (IllegalStateException unused) {
                }
            }
            Thread thread = this.worker;
            if (thread != null) {
                thread.join(1000L);
            }
            if (audioRecord != null) {
                audioRecord.release();
            }
            this.worker = null;
            this.recorder = null;
        }
    }
}
