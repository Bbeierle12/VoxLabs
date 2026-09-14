package org.vocaltract.pixel;

import java.io.BufferedInputStream;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.List;
import kotlin.Metadata;
import kotlin.UShort;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.jvm.internal.Intrinsics;
import kotlin.text.Charsets;

/* compiled from: ReplaySources.kt */
@Metadata(d1 = {"\u0000&\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u0012\n\u0002\b\u0006\bÆ\u0002\u0018\u00002\u00020\u0001:\u0001\u0011B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u0016\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u00072\u0006\u0010\b\u001a\u00020\tJ\u0018\u0010\n\u001a\u00020\u000b2\u0006\u0010\u0006\u001a\u00020\u00072\u0006\u0010\f\u001a\u00020\tH\u0002J\u0018\u0010\r\u001a\u00020\t2\u0006\u0010\u000e\u001a\u00020\u000b2\u0006\u0010\u000f\u001a\u00020\tH\u0002J\u0018\u0010\u0010\u001a\u00020\t2\u0006\u0010\u000e\u001a\u00020\u000b2\u0006\u0010\u000f\u001a\u00020\tH\u0002"}, d2 = {"Lorg/vocaltract/pixel/WavPcmReplay;", "", "<init>", "()V", "read", "Lorg/vocaltract/pixel/WavPcmReplay$WavFrames;", "input", "Ljava/io/InputStream;", "frameSize", "", "readExactly", "", "size", "littleInt", "bytes", "offset", "littleShort", "WavFrames"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class WavPcmReplay {
    public static final WavPcmReplay INSTANCE = new WavPcmReplay();

    /* compiled from: ReplaySources.kt */
    @Metadata(d1 = {"\u0000(\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0010 \n\u0002\u0010\u0017\n\u0002\b\n\n\u0002\u0010\u000b\n\u0002\b\u0003\n\u0002\u0010\u000e\b\u0086\b\u0018\u00002\u00020\u0001B\u001d\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\f\u0010\u0004\u001a\b\u0012\u0004\u0012\u00020\u00060\u0005¢\u0006\u0004\b\u0007\u0010\bJ\t\u0010\r\u001a\u00020\u0003HÆ\u0003J\u000f\u0010\u000e\u001a\b\u0012\u0004\u0012\u00020\u00060\u0005HÆ\u0003J#\u0010\u000f\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\u000e\b\u0002\u0010\u0004\u001a\b\u0012\u0004\u0012\u00020\u00060\u0005HÆ\u0001J\u0013\u0010\u0010\u001a\u00020\u00112\b\u0010\u0012\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0013\u001a\u00020\u0003HÖ\u0001J\t\u0010\u0014\u001a\u00020\u0015HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\t\u0010\nR\u0017\u0010\u0004\u001a\b\u0012\u0004\u0012\u00020\u00060\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\f"}, d2 = {"Lorg/vocaltract/pixel/WavPcmReplay$WavFrames;", "", "sampleRateHz", "", "frames", "", "", "<init>", "(ILjava/util/List;)V", "getSampleRateHz", "()I", "getFrames", "()Ljava/util/List;", "component1", "component2", "copy", "equals", "", "other", "hashCode", "toString", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final /* data */ class WavFrames {
        private final List<short[]> frames;
        private final int sampleRateHz;

        /* JADX DEBUG: Multi-variable search result rejected for r0v0, resolved type: org.vocaltract.pixel.WavPcmReplay$WavFrames */
        /* JADX WARN: Multi-variable type inference failed */
        public static /* synthetic */ WavFrames copy$default(WavFrames wavFrames, int i, List list, int i2, Object obj) {
            if ((i2 & 1) != 0) {
                i = wavFrames.sampleRateHz;
            }
            if ((i2 & 2) != 0) {
                list = wavFrames.frames;
            }
            return wavFrames.copy(i, list);
        }

        /* renamed from: component1, reason: from getter */
        public final int getSampleRateHz() {
            return this.sampleRateHz;
        }

        public final List<short[]> component2() {
            return this.frames;
        }

        public final WavFrames copy(int sampleRateHz, List<short[]> frames) {
            Intrinsics.checkNotNullParameter(frames, "frames");
            return new WavFrames(sampleRateHz, frames);
        }

        public boolean equals(Object other) {
            if (this == other) {
                return true;
            }
            if (!(other instanceof WavFrames)) {
                return false;
            }
            WavFrames wavFrames = (WavFrames) other;
            return this.sampleRateHz == wavFrames.sampleRateHz && Intrinsics.areEqual(this.frames, wavFrames.frames);
        }

        public int hashCode() {
            return (Integer.hashCode(this.sampleRateHz) * 31) + this.frames.hashCode();
        }

        public String toString() {
            return "WavFrames(sampleRateHz=" + this.sampleRateHz + ", frames=" + this.frames + ")";
        }

        public WavFrames(int i, List<short[]> frames) {
            Intrinsics.checkNotNullParameter(frames, "frames");
            this.sampleRateHz = i;
            this.frames = frames;
        }

        public final List<short[]> getFrames() {
            return this.frames;
        }

        public final int getSampleRateHz() {
            return this.sampleRateHz;
        }
    }

    private WavPcmReplay() {
    }

    public final WavFrames read(InputStream input, int frameSize) {
        Intrinsics.checkNotNullParameter(input, "input");
        if (frameSize <= 0) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        BufferedInputStream bufferedInputStream = new BufferedInputStream(input);
        BufferedInputStream bufferedInputStream2 = bufferedInputStream;
        byte[] readExactly = readExactly(bufferedInputStream2, 12);
        int i = 4;
        if (!Intrinsics.areEqual(new String(readExactly, 0, 4, Charsets.US_ASCII), "RIFF")) {
            throw new IllegalArgumentException("Not a RIFF file".toString());
        }
        int i2 = 8;
        if (!Intrinsics.areEqual(new String(readExactly, 8, 4, Charsets.US_ASCII), "WAVE")) {
            throw new IllegalArgumentException("Not a WAVE file".toString());
        }
        byte[] bArr = null;
        int i3 = 0;
        int i4 = 0;
        int i5 = 0;
        int i6 = 0;
        while (bArr == null) {
            byte[] readExactly2 = readExactly(bufferedInputStream2, i2);
            String str = new String(readExactly2, 0, i, Charsets.US_ASCII);
            int littleInt = littleInt(readExactly2, i);
            if (littleInt < 0) {
                throw new IllegalArgumentException("Invalid WAV chunk size".toString());
            }
            byte[] readExactly3 = readExactly(bufferedInputStream2, littleInt);
            if ((littleInt & 1) == 1) {
                bufferedInputStream.read();
            }
            if (Intrinsics.areEqual(str, "fmt ")) {
                if (littleInt < 16) {
                    throw new IllegalArgumentException("Truncated WAV format chunk".toString());
                }
                i3 = littleShort(readExactly3, 0);
                i5 = littleShort(readExactly3, 2);
                i = 4;
                i6 = littleInt(readExactly3, 4);
                i4 = littleShort(readExactly3, 14);
            } else {
                i = 4;
                if (Intrinsics.areEqual(str, "data")) {
                    bArr = readExactly3;
                }
            }
            i2 = 8;
        }
        if (i3 != 1 || i4 != 16 || 1 > i5 || i5 >= 3) {
            throw new IllegalArgumentException("Only mono/stereo 16-bit PCM WAV is supported".toString());
        }
        if (i6 <= 0) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        int length = (bArr.length / 2) / i5;
        short[] sArr = new short[length];
        ByteBuffer order = ByteBuffer.wrap(bArr).order(ByteOrder.LITTLE_ENDIAN);
        for (int i7 = 0; i7 < length; i7++) {
            int i8 = 0;
            for (int i9 = 0; i9 < i5; i9++) {
                i8 += order.getShort();
            }
            sArr[i7] = (short) (i8 / i5);
        }
        List<List> chunked = CollectionsKt.chunked(ArraysKt.asList(sArr), frameSize);
        ArrayList arrayList = new ArrayList(CollectionsKt.collectionSizeOrDefault(chunked, 10));
        for (List list : chunked) {
            short[] sArr2 = new short[frameSize];
            int i10 = 0;
            while (i10 < frameSize) {
                sArr2[i10] = i10 < list.size() ? ((Number) list.get(i10)).shortValue() : (short) 0;
                i10++;
            }
            arrayList.add(sArr2);
        }
        return new WavFrames(i6, arrayList);
    }

    private final byte[] readExactly(InputStream input, int size) {
        byte[] bArr = new byte[size];
        int i = 0;
        while (i < size) {
            int read = input.read(bArr, i, size - i);
            if (read < 0) {
                throw new IllegalArgumentException("Unexpected end of WAV stream".toString());
            }
            i += read;
        }
        return bArr;
    }

    private final int littleInt(byte[] bytes, int offset) {
        return ByteBuffer.wrap(bytes, offset, 4).order(ByteOrder.LITTLE_ENDIAN).getInt();
    }

    private final int littleShort(byte[] bytes, int offset) {
        return ByteBuffer.wrap(bytes, offset, 2).order(ByteOrder.LITTLE_ENDIAN).getShort() & UShort.MAX_VALUE;
    }
}
