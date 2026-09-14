package org.vocaltract.pixel;

import android.opengl.GLES30;
import android.opengl.GLSurfaceView;
import android.opengl.Matrix;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;
import java.util.Arrays;
import javax.microedition.khronos.egl.EGLConfig;
import javax.microedition.khronos.opengles.GL10;
import kotlin.Metadata;
import kotlin.Unit;
import kotlin.collections.ArraysKt;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: TractMeshRenderer.kt */
@Metadata(d1 = {"\u0000z\n\u0002\u0018\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\u0010\u000e\n\u0002\u0010\u0002\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\u0015\n\u0002\b\u0002\n\u0002\u0010\u0014\n\u0002\b\u0002\n\u0002\u0010\u0007\n\u0002\b\u0005\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0010\t\n\u0002\b\u0011\n\u0002\u0018\u0002\n\u0002\b\f\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b%\b\u0000\u0018\u0000 l2\u00020\u0001:\u0002klB\u001b\u0012\u0012\u0010\u0002\u001a\u000e\u0012\u0004\u0012\u00020\u0004\u0012\u0004\u0012\u00020\u00050\u0003¢\u0006\u0004\b\u0006\u0010\u0007J\u000e\u00105\u001a\u00020\u00052\u0006\u00106\u001a\u00020\u000eJ\u000e\u00107\u001a\u00020\u00052\u0006\u00106\u001a\u000208J\u000e\u00109\u001a\u00020\u00052\u0006\u00106\u001a\u00020\u000bJ\u0016\u0010:\u001a\u00020\u00052\u0006\u0010;\u001a\u00020\u001b2\u0006\u0010<\u001a\u00020\u001bJ\u000e\u0010=\u001a\u00020\u00052\u0006\u0010>\u001a\u00020\u001bJ\u0006\u0010?\u001a\u00020\u0005J\u000e\u0010@\u001a\u00020\u00052\u0006\u0010A\u001a\u00020\u0012J\u000e\u0010B\u001a\u00020\u00052\u0006\u0010A\u001a\u00020\u0012J\u001c\u0010C\u001a\u00020\u00052\b\u0010D\u001a\u0004\u0018\u00010E2\b\u0010F\u001a\u0004\u0018\u00010GH\u0016J\"\u0010H\u001a\u00020\u00052\b\u0010D\u001a\u0004\u0018\u00010E2\u0006\u0010I\u001a\u00020\u00102\u0006\u0010J\u001a\u00020\u0010H\u0016J\u0012\u0010K\u001a\u00020\u00052\b\u0010D\u001a\u0004\u0018\u00010EH\u0016J\u0010\u0010L\u001a\u00020\u00052\u0006\u00106\u001a\u00020\u000eH\u0002J\u0010\u0010M\u001a\u00020\u00052\u0006\u0010N\u001a\u00020\u000eH\u0002J\u0018\u0010O\u001a\u00020\u00052\u0006\u0010N\u001a\u00020\u000e2\u0006\u0010P\u001a\u00020\u001bH\u0002J@\u0010Q\u001a\u00020\u00052\u0006\u0010R\u001a\u00020\u00102\u0006\u0010S\u001a\u00020\u00102\u0006\u0010T\u001a\u00020\u00102\u0006\u0010U\u001a\u00020\u00102\u0006\u0010V\u001a\u00020\u00182\u0006\u0010W\u001a\u00020\u001b2\u0006\u0010X\u001a\u00020&H\u0002J\b\u0010Y\u001a\u00020\u0005H\u0002J\b\u0010Z\u001a\u00020\u0005H\u0002J\u0010\u0010[\u001a\u00020\u00052\u0006\u0010\\\u001a\u00020\u0018H\u0002J\u0018\u0010]\u001a\u00020\u00052\u0006\u0010^\u001a\u00020\u00102\u0006\u0010_\u001a\u00020\u0015H\u0002J\u0010\u0010`\u001a\u00020!2\u0006\u0010T\u001a\u00020\u0010H\u0002J \u0010a\u001a\u00020\u00052\u0006\u0010^\u001a\u00020\u00102\u0006\u0010b\u001a\u00020!2\u0006\u0010_\u001a\u00020\u0018H\u0002J\u0018\u0010c\u001a\u00020\u00102\u0006\u0010d\u001a\u00020\u00042\u0006\u0010e\u001a\u00020\u0004H\u0002J\u0018\u0010f\u001a\u00020\u00102\u0006\u0010g\u001a\u00020\u00102\u0006\u0010h\u001a\u00020\u0004H\u0002J\u0010\u0010i\u001a\u00020\u00052\u0006\u0010j\u001a\u00020\u0004H\u0002R\u001a\u0010\u0002\u001a\u000e\u0012\u0004\u0012\u00020\u0004\u0012\u0004\u0012\u00020\u00050\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u0010\u0010\b\u001a\u0004\u0018\u00010\tX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\u000bX\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\f\u001a\u0004\u0018\u00010\tX\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\r\u001a\u0004\u0018\u00010\u000eX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u000f\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0011\u001a\u00020\u0012X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0013\u001a\u00020\u0012X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0014\u001a\u00020\u0015X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0016\u001a\u00020\u0015X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0017\u001a\u00020\u0018X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0019\u001a\u00020\u0018X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001a\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001c\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001d\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001e\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u001f\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010 \u001a\u0004\u0018\u00010!X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\"\u001a\u0004\u0018\u00010!X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010#\u001a\u0004\u0018\u00010!X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010$\u001a\u00020\u0012X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010%\u001a\u00020&X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010'\u001a\u00020\u0018X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010(\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010)\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010*\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010+\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010,\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010-\u001a\u00020\u001bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010.\u001a\u00020\u0012X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010/\u001a\u00020\u0012X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u00100\u001a\u00020\u0018X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u00101\u001a\u00020\u0018X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u00102\u001a\u00020\u0018X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u00103\u001a\u00020\u0018X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u00104\u001a\u00020\u0018X\u0082\u0004¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/TractMeshRenderer;", "Landroid/opengl/GLSurfaceView$Renderer;", "reportFailure", "Lkotlin/Function1;", "", "", "<init>", "(Lkotlin/jvm/functions/Function1;)V", "pendingPosterior", "Lorg/vocaltract/pixel/TractMeshRenderer$PosteriorTarget;", "pendingOverlay", "Lorg/vocaltract/pixel/AcousticMatchSummary;", "consumedPosterior", "asset", "Lorg/vocaltract/pixel/TractLumenAsset;", "program", "", "glReady", "", "failureReported", "buffers", "", "lineIndices", "currentArea", "", "targetArea", "currentConfidence", "", "targetConfidence", "currentRelativeStd", "targetRelativeStd", "currentMatch", "positionUpload", "Ljava/nio/FloatBuffer;", "normalUpload", "shellUpload", "geometryDirty", "lastFrameNanos", "", "meshCenter", "meshScale", "surfaceWidth", "surfaceHeight", "pitchDegrees", "yawDegrees", "zoom", "cutawayEnabled", "wireframeEnabled", "projection", "view", "model", "viewProjection", "mvp", "setMeshAsset", "value", "submitPosterior", "Lorg/vocaltract/pixel/TractPosterior;", "submitOverlay", "rotateBy", "deltaPitchDegrees", "deltaYawDegrees", "zoomBy", "scaleFactor", "resetCamera", "setCutawayEnabled", "enabled", "setWireframeEnabled", "onSurfaceCreated", "unused", "Ljavax/microedition/khronos/opengles/GL10;", "config", "Ljavax/microedition/khronos/egl/EGLConfig;", "onSurfaceChanged", "width", "height", "onDrawFrame", "configureMesh", "updateTargets", "activeAsset", "smoothAndUpload", "dt", "draw", "positionBuffer", "indexBuffer", "count", "primitive", "color", "match", "nowNanos", "updateProjection", "updateMatrices", "calculateBounds", "vertices", "uploadIndices", "buffer", "values", "allocateFloatBuffer", "uploadFloats", "upload", "createProgram", "vertexSource", "fragmentSource", "compileShader", "type", "source", "fail", "message", "PosteriorTarget", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class TractMeshRenderer implements GLSurfaceView.Renderer {
    private static final int LINE_INDEX_BUFFER = 4;
    private static final int NORMAL_ATTRIBUTE = 1;
    private static final int NORMAL_BUFFER = 1;
    private static final int POSITION_ATTRIBUTE = 0;
    private static final int POSITION_BUFFER = 0;
    private static final int SHELL_BUFFER = 2;
    private static final int TRIANGLE_INDEX_BUFFER = 3;
    private TractLumenAsset asset;
    private final int[] buffers;
    private PosteriorTarget consumedPosterior;
    private float[] currentArea;
    private float currentConfidence;
    private float currentMatch;
    private float currentRelativeStd;
    private boolean cutawayEnabled;
    private boolean failureReported;
    private boolean geometryDirty;
    private boolean glReady;
    private long lastFrameNanos;
    private int[] lineIndices;
    private final float[] meshCenter;
    private float meshScale;
    private final float[] model;
    private final float[] mvp;
    private FloatBuffer normalUpload;
    private volatile AcousticMatchSummary pendingOverlay;
    private volatile PosteriorTarget pendingPosterior;
    private float pitchDegrees;
    private FloatBuffer positionUpload;
    private int program;
    private final float[] projection;
    private final Function1<String, Unit> reportFailure;
    private FloatBuffer shellUpload;
    private int surfaceHeight;
    private int surfaceWidth;
    private float[] targetArea;
    private float targetConfidence;
    private float targetRelativeStd;
    private final float[] view;
    private final float[] viewProjection;
    private boolean wireframeEnabled;
    private float yawDegrees;
    private float zoom;
    private static final String VERTEX_SHADER = "#version 300 es\nlayout(location = 0) in vec3 aPosition;\nlayout(location = 1) in vec3 aNormal;\nuniform mat4 uMvp;\nuniform mat4 uModel;\nout vec3 vNormal;\nout vec3 vLocalPosition;\nvoid main() {\n    vLocalPosition = aPosition;\n    vNormal = normalize(mat3(uModel) * aNormal);\n    gl_Position = uMvp * vec4(aPosition, 1.0);\n}";
    private static final String FRAGMENT_SHADER = "#version 300 es\nprecision mediump float;\nin vec3 vNormal;\nin vec3 vLocalPosition;\nuniform vec4 uBaseColor;\nuniform vec3 uLightDirection;\nuniform float uCutaway;\nuniform float uCutPlane;\nuniform float uMatch;\nuniform float uPulse;\nout vec4 fragmentColor;\nvoid main() {\n    if (uCutaway > 0.5 && vLocalPosition.z > uCutPlane) discard;\n    float diffuse = 0.25 + 0.75 * abs(dot(normalize(vNormal), normalize(uLightDirection)));\n    vec3 matchColor = vec3(0.98, 0.36, 0.12);\n    vec3 tint = mix(uBaseColor.rgb, matchColor, clamp(uMatch * uPulse * 0.68, 0.0, 0.68));\n    fragmentColor = vec4(tint * diffuse, uBaseColor.a);\n}";

    /* JADX DEBUG: Multi-variable search result rejected for r2v0, resolved type: kotlin.jvm.functions.Function1<? super java.lang.String, kotlin.Unit> */
    /* JADX WARN: Multi-variable type inference failed */
    public TractMeshRenderer(Function1<? super String, Unit> reportFailure) {
        Intrinsics.checkNotNullParameter(reportFailure, "reportFailure");
        this.reportFailure = reportFailure;
        this.pendingOverlay = AcousticMatchSummary.INSTANCE.getNONE();
        this.buffers = new int[5];
        this.lineIndices = new int[0];
        this.currentArea = new float[0];
        this.targetArea = new float[0];
        this.currentConfidence = 0.5f;
        this.targetConfidence = 0.5f;
        this.currentRelativeStd = 0.15f;
        this.targetRelativeStd = 0.15f;
        this.geometryDirty = true;
        this.meshCenter = new float[3];
        this.meshScale = 1.0f;
        this.surfaceWidth = 1;
        this.surfaceHeight = 1;
        this.pitchDegrees = -14.0f;
        this.yawDegrees = 24.0f;
        this.zoom = 1.0f;
        this.projection = new float[16];
        this.view = new float[16];
        this.model = new float[16];
        this.viewProjection = new float[16];
        this.mvp = new float[16];
    }

    /* compiled from: TractMeshRenderer.kt */
    @Metadata(d1 = {"\u0000,\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0014\n\u0002\b\u0002\n\u0002\u0010\u0007\n\u0002\b\u000f\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u000e\b\u0082\b\u0018\u00002\u00020\u0001B'\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0006\u0012\u0006\u0010\u0007\u001a\u00020\u0006¢\u0006\u0004\b\b\u0010\tJ\t\u0010\u0010\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0011\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0012\u001a\u00020\u0006HÆ\u0003J\t\u0010\u0013\u001a\u00020\u0006HÆ\u0003J1\u0010\u0014\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u00062\b\b\u0002\u0010\u0007\u001a\u00020\u0006HÆ\u0001J\u0013\u0010\u0015\u001a\u00020\u00162\b\u0010\u0017\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0018\u001a\u00020\u0019HÖ\u0001J\t\u0010\u001a\u001a\u00020\u001bHÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\n\u0010\u000bR\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\f\u0010\u000bR\u0011\u0010\u0005\u001a\u00020\u0006¢\u0006\b\n\u0000\u001a\u0004\b\r\u0010\u000eR\u0011\u0010\u0007\u001a\u00020\u0006¢\u0006\b\n\u0000\u001a\u0004\b\u000f\u0010\u000e"}, d2 = {"Lorg/vocaltract/pixel/TractMeshRenderer$PosteriorTarget;", "", "sectionPosition", "", "areaCm2", "confidence", "", "relativeAreaStd", "<init>", "([F[FFF)V", "getSectionPosition", "()[F", "getAreaCm2", "getConfidence", "()F", "getRelativeAreaStd", "component1", "component2", "component3", "component4", "copy", "equals", "", "other", "hashCode", "", "toString", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
    private static final /* data */ class PosteriorTarget {
        private final float[] areaCm2;
        private final float confidence;
        private final float relativeAreaStd;
        private final float[] sectionPosition;

        public static /* synthetic */ PosteriorTarget copy$default(PosteriorTarget posteriorTarget, float[] fArr, float[] fArr2, float f, float f2, int i, Object obj) {
            if ((i & 1) != 0) {
                fArr = posteriorTarget.sectionPosition;
            }
            if ((i & 2) != 0) {
                fArr2 = posteriorTarget.areaCm2;
            }
            if ((i & 4) != 0) {
                f = posteriorTarget.confidence;
            }
            if ((i & 8) != 0) {
                f2 = posteriorTarget.relativeAreaStd;
            }
            return posteriorTarget.copy(fArr, fArr2, f, f2);
        }

        /* renamed from: component1, reason: from getter */
        public final float[] getSectionPosition() {
            return this.sectionPosition;
        }

        /* renamed from: component2, reason: from getter */
        public final float[] getAreaCm2() {
            return this.areaCm2;
        }

        /* renamed from: component3, reason: from getter */
        public final float getConfidence() {
            return this.confidence;
        }

        /* renamed from: component4, reason: from getter */
        public final float getRelativeAreaStd() {
            return this.relativeAreaStd;
        }

        public final PosteriorTarget copy(float[] sectionPosition, float[] areaCm2, float confidence, float relativeAreaStd) {
            Intrinsics.checkNotNullParameter(sectionPosition, "sectionPosition");
            Intrinsics.checkNotNullParameter(areaCm2, "areaCm2");
            return new PosteriorTarget(sectionPosition, areaCm2, confidence, relativeAreaStd);
        }

        public boolean equals(Object other) {
            if (this == other) {
                return true;
            }
            if (!(other instanceof PosteriorTarget)) {
                return false;
            }
            PosteriorTarget posteriorTarget = (PosteriorTarget) other;
            return Intrinsics.areEqual(this.sectionPosition, posteriorTarget.sectionPosition) && Intrinsics.areEqual(this.areaCm2, posteriorTarget.areaCm2) && Float.compare(this.confidence, posteriorTarget.confidence) == 0 && Float.compare(this.relativeAreaStd, posteriorTarget.relativeAreaStd) == 0;
        }

        public int hashCode() {
            return (((((Arrays.hashCode(this.sectionPosition) * 31) + Arrays.hashCode(this.areaCm2)) * 31) + Float.hashCode(this.confidence)) * 31) + Float.hashCode(this.relativeAreaStd);
        }

        public String toString() {
            return "PosteriorTarget(sectionPosition=" + Arrays.toString(this.sectionPosition) + ", areaCm2=" + Arrays.toString(this.areaCm2) + ", confidence=" + this.confidence + ", relativeAreaStd=" + this.relativeAreaStd + ")";
        }

        public PosteriorTarget(float[] sectionPosition, float[] areaCm2, float f, float f2) {
            Intrinsics.checkNotNullParameter(sectionPosition, "sectionPosition");
            Intrinsics.checkNotNullParameter(areaCm2, "areaCm2");
            this.sectionPosition = sectionPosition;
            this.areaCm2 = areaCm2;
            this.confidence = f;
            this.relativeAreaStd = f2;
        }

        public final float[] getSectionPosition() {
            return this.sectionPosition;
        }

        public final float[] getAreaCm2() {
            return this.areaCm2;
        }

        public final float getConfidence() {
            return this.confidence;
        }

        public final float getRelativeAreaStd() {
            return this.relativeAreaStd;
        }
    }

    public final void setMeshAsset(TractLumenAsset value) {
        Intrinsics.checkNotNullParameter(value, "value");
        this.asset = value;
        if (this.glReady) {
            configureMesh(value);
        }
    }

    public final void submitPosterior(TractPosterior value) {
        Intrinsics.checkNotNullParameter(value, "value");
        float[] sectionPosition = value.getSectionPosition();
        float[] copyOf = Arrays.copyOf(sectionPosition, sectionPosition.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        float[] areaCm2 = value.getAreaCm2();
        float[] copyOf2 = Arrays.copyOf(areaCm2, areaCm2.length);
        Intrinsics.checkNotNullExpressionValue(copyOf2, "copyOf(...)");
        this.pendingPosterior = new PosteriorTarget(copyOf, copyOf2, RangesKt.coerceIn(value.getConfidence(), 0.0f, 1.0f), RangesKt.coerceIn(value.getRelativeAreaStd(), 0.0f, 1.5f));
    }

    public final void submitOverlay(AcousticMatchSummary value) {
        Intrinsics.checkNotNullParameter(value, "value");
        this.pendingOverlay = value;
    }

    public final void rotateBy(float deltaPitchDegrees, float deltaYawDegrees) {
        this.pitchDegrees = RangesKt.coerceIn(this.pitchDegrees + deltaPitchDegrees, -85.0f, 85.0f);
        this.yawDegrees = (this.yawDegrees + deltaYawDegrees) % 360.0f;
    }

    public final void zoomBy(float scaleFactor) {
        this.zoom = RangesKt.coerceIn(this.zoom * RangesKt.coerceIn(scaleFactor, 0.75f, 1.33f), 0.55f, 3.2f);
    }

    public final void resetCamera() {
        this.pitchDegrees = -14.0f;
        this.yawDegrees = 24.0f;
        this.zoom = 1.0f;
    }

    public final void setCutawayEnabled(boolean enabled) {
        this.cutawayEnabled = enabled;
    }

    public final void setWireframeEnabled(boolean enabled) {
        this.wireframeEnabled = enabled;
    }

    @Override // android.opengl.GLSurfaceView.Renderer
    public void onSurfaceCreated(GL10 unused, EGLConfig config) {
        try {
            this.program = createProgram(VERTEX_SHADER, FRAGMENT_SHADER);
            GLES30.glClearColor(0.969f, 0.988f, 0.98f, 1.0f);
            GLES30.glEnable(2929);
            GLES30.glDepthFunc(515);
            GLES30.glEnable(3042);
            GLES30.glBlendFunc(770, 771);
            this.glReady = true;
            ArraysKt.fill$default(this.buffers, 0, 0, 0, 6, (Object) null);
            TractLumenAsset tractLumenAsset = this.asset;
            if (tractLumenAsset != null) {
                configureMesh(tractLumenAsset);
            }
        } catch (RuntimeException e) {
            this.glReady = false;
            String message = e.getMessage();
            if (message == null) {
                message = e.getClass().getSimpleName();
            }
            fail("3D renderer unavailable: " + message);
        }
    }

    @Override // android.opengl.GLSurfaceView.Renderer
    public void onSurfaceChanged(GL10 unused, int width, int height) {
        this.surfaceWidth = Math.max(1, width);
        int max = Math.max(1, height);
        this.surfaceHeight = max;
        GLES30.glViewport(0, 0, this.surfaceWidth, max);
        updateProjection();
    }

    @Override // android.opengl.GLSurfaceView.Renderer
    public void onDrawFrame(GL10 unused) {
        GLES30.glClear(16640);
        TractLumenAsset tractLumenAsset = this.asset;
        if (tractLumenAsset == null || !this.glReady || this.program == 0 || this.buffers[0] == 0) {
            return;
        }
        long nanoTime = System.nanoTime();
        float coerceIn = this.lastFrameNanos == 0 ? 0.016666668f : RangesKt.coerceIn((float) ((nanoTime - r0) / 1.0E9d), 0.004166667f, 0.1f);
        this.lastFrameNanos = nanoTime;
        updateTargets(tractLumenAsset);
        smoothAndUpload(tractLumenAsset, coerceIn);
        updateMatrices();
        if (this.cutawayEnabled) {
            GLES30.glDisable(2884);
        } else {
            GLES30.glEnable(2884);
            GLES30.glCullFace(1029);
        }
        float f = ((1.0f - this.currentConfidence) * 0.24f) + 0.08f;
        GLES30.glDepthMask(false);
        int[] iArr = this.buffers;
        draw(iArr[2], iArr[3], tractLumenAsset.getTriangleIndices().length, 4, new float[]{0.23f, 0.72f, 0.66f, f}, this.currentMatch * 0.45f, nanoTime);
        GLES30.glDepthMask(true);
        float f2 = 1.0f - this.currentConfidence;
        int[] iArr2 = this.buffers;
        draw(iArr2[0], iArr2[3], tractLumenAsset.getTriangleIndices().length, 4, new float[]{(0.55f * f2) + 0.035f, 0.6f - (0.18f * f2), 0.53f - (f2 * 0.28f), 1.0f}, this.currentMatch, nanoTime);
        if (this.wireframeEnabled) {
            if (!(this.lineIndices.length == 0)) {
                GLES30.glDisable(2884);
                GLES30.glLineWidth(1.0f);
                int[] iArr3 = this.buffers;
                draw(iArr3[0], iArr3[4], this.lineIndices.length, 1, new float[]{0.015f, 0.1f, 0.09f, 0.72f}, 0.0f, nanoTime);
            }
        }
    }

    private final void configureMesh(TractLumenAsset value) {
        int[] iArr = this.buffers;
        int length = iArr.length;
        int i = 0;
        while (true) {
            if (i >= length) {
                break;
            }
            if (iArr[i] != 0) {
                int[] iArr2 = this.buffers;
                GLES30.glDeleteBuffers(iArr2.length, iArr2, 0);
                break;
            }
            i++;
        }
        int[] iArr3 = this.buffers;
        GLES30.glGenBuffers(iArr3.length, iArr3, 0);
        this.lineIndices = TractMeshCpu.INSTANCE.buildLineIndices(value.getTriangleIndices());
        float[] referenceAreaCm2 = value.getReferenceAreaCm2();
        float[] copyOf = Arrays.copyOf(referenceAreaCm2, referenceAreaCm2.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        this.currentArea = copyOf;
        float[] copyOf2 = Arrays.copyOf(copyOf, copyOf.length);
        Intrinsics.checkNotNullExpressionValue(copyOf2, "copyOf(...)");
        this.targetArea = copyOf2;
        this.currentConfidence = 0.5f;
        this.targetConfidence = 0.5f;
        this.currentRelativeStd = 0.15f;
        this.targetRelativeStd = 0.15f;
        this.consumedPosterior = null;
        calculateBounds(value.getMeanVertices());
        int length2 = value.getMeanVertices().length * 4;
        this.positionUpload = allocateFloatBuffer(value.getMeanVertices().length);
        this.normalUpload = allocateFloatBuffer(value.getMeanVertices().length);
        this.shellUpload = allocateFloatBuffer(value.getMeanVertices().length);
        int[] iArr4 = this.buffers;
        int[] iArr5 = {iArr4[0], iArr4[1], iArr4[2]};
        for (int i2 = 0; i2 < 3; i2++) {
            GLES30.glBindBuffer(34962, iArr5[i2]);
            GLES30.glBufferData(34962, length2, null, 35048);
        }
        uploadIndices(this.buffers[3], value.getTriangleIndices());
        uploadIndices(this.buffers[4], this.lineIndices);
        GLES30.glBindBuffer(34962, 0);
        GLES30.glBindBuffer(34963, 0);
        this.geometryDirty = true;
        smoothAndUpload(value, 1.0f);
    }

    private final void updateTargets(TractLumenAsset activeAsset) {
        float[] copyOf;
        PosteriorTarget posteriorTarget = this.pendingPosterior;
        if (posteriorTarget == null || posteriorTarget == this.consumedPosterior) {
            return;
        }
        if (posteriorTarget.getSectionPosition().length == posteriorTarget.getAreaCm2().length) {
            if (!(posteriorTarget.getSectionPosition().length == 0)) {
                try {
                    copyOf = TractMeshCpu.INSTANCE.resampleArea(posteriorTarget.getSectionPosition(), posteriorTarget.getAreaCm2(), activeAsset.getSectionPosition());
                } catch (IllegalArgumentException unused) {
                    float[] referenceAreaCm2 = activeAsset.getReferenceAreaCm2();
                    copyOf = Arrays.copyOf(referenceAreaCm2, referenceAreaCm2.length);
                    Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
                }
                this.targetArea = copyOf;
                this.targetConfidence = posteriorTarget.getConfidence();
                this.targetRelativeStd = posteriorTarget.getRelativeAreaStd();
                this.consumedPosterior = posteriorTarget;
            }
        }
        float[] referenceAreaCm22 = activeAsset.getReferenceAreaCm2();
        copyOf = Arrays.copyOf(referenceAreaCm22, referenceAreaCm22.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        this.targetArea = copyOf;
        this.targetConfidence = posteriorTarget.getConfidence();
        this.targetRelativeStd = posteriorTarget.getRelativeAreaStd();
        this.consumedPosterior = posteriorTarget;
    }

    private final void smoothAndUpload(TractLumenAsset activeAsset, float dt) {
        float coerceIn = RangesKt.coerceIn((float) (1.0d - Math.exp((-dt) / 0.11f)), 0.0f, 1.0f);
        boolean z = this.geometryDirty;
        int length = this.currentArea.length;
        for (int i = 0; i < length; i++) {
            float f = this.targetArea[i] - this.currentArea[i];
            if (Math.abs(f) > 1.0E-4f) {
                float[] fArr = this.currentArea;
                fArr[i] = fArr[i] + (f * coerceIn);
                z = true;
            }
        }
        float f2 = this.targetRelativeStd - this.currentRelativeStd;
        if (Math.abs(f2) > 1.0E-4f) {
            this.currentRelativeStd += f2 * coerceIn;
            z = true;
        }
        float f3 = this.currentConfidence;
        this.currentConfidence = f3 + ((this.targetConfidence - f3) * coerceIn);
        Float valueOf = Float.valueOf(this.pendingOverlay.getScore());
        valueOf.floatValue();
        if (!this.pendingOverlay.getActive()) {
            valueOf = null;
        }
        float floatValue = valueOf != null ? valueOf.floatValue() : 0.0f;
        float f4 = this.currentMatch;
        this.currentMatch = f4 + ((floatValue - f4) * coerceIn);
        if (z) {
            CpuTractFrame prepare = TractMeshCpu.INSTANCE.prepare(activeAsset, this.currentArea, this.currentRelativeStd);
            int i2 = this.buffers[0];
            FloatBuffer floatBuffer = this.positionUpload;
            Intrinsics.checkNotNull(floatBuffer);
            uploadFloats(i2, floatBuffer, prepare.getVertices());
            int i3 = this.buffers[1];
            FloatBuffer floatBuffer2 = this.normalUpload;
            Intrinsics.checkNotNull(floatBuffer2);
            uploadFloats(i3, floatBuffer2, prepare.getNormals());
            int i4 = this.buffers[2];
            FloatBuffer floatBuffer3 = this.shellUpload;
            Intrinsics.checkNotNull(floatBuffer3);
            uploadFloats(i4, floatBuffer3, prepare.getUncertaintyVertices());
            this.geometryDirty = false;
        }
    }

    private final void draw(int positionBuffer, int indexBuffer, int count, int primitive, float[] color, float match, long nowNanos) {
        GLES30.glUseProgram(this.program);
        GLES30.glUniformMatrix4fv(GLES30.glGetUniformLocation(this.program, "uMvp"), 1, false, this.mvp, 0);
        GLES30.glUniformMatrix4fv(GLES30.glGetUniformLocation(this.program, "uModel"), 1, false, this.model, 0);
        GLES30.glUniform4fv(GLES30.glGetUniformLocation(this.program, "uBaseColor"), 1, color, 0);
        GLES30.glUniform3f(GLES30.glGetUniformLocation(this.program, "uLightDirection"), -0.35f, 0.55f, 0.76f);
        GLES30.glUniform1f(GLES30.glGetUniformLocation(this.program, "uCutaway"), this.cutawayEnabled ? 1.0f : 0.0f);
        GLES30.glUniform1f(GLES30.glGetUniformLocation(this.program, "uCutPlane"), this.meshCenter[2]);
        GLES30.glUniform1f(GLES30.glGetUniformLocation(this.program, "uMatch"), RangesKt.coerceIn(match, 0.0f, 1.0f));
        GLES30.glUniform1f(GLES30.glGetUniformLocation(this.program, "uPulse"), (((float) Math.sin(((nowNanos % 10000000000L) / 1.0E9f) * 6.4f)) * 0.3f) + 0.7f);
        GLES30.glBindBuffer(34962, positionBuffer);
        GLES30.glEnableVertexAttribArray(0);
        GLES30.glVertexAttribPointer(0, 3, 5126, false, 0, 0);
        GLES30.glBindBuffer(34962, this.buffers[1]);
        GLES30.glEnableVertexAttribArray(1);
        GLES30.glVertexAttribPointer(1, 3, 5126, false, 0, 0);
        GLES30.glBindBuffer(34963, indexBuffer);
        GLES30.glDrawElements(primitive, count, 5125, 0);
    }

    private final void updateProjection() {
        Matrix.perspectiveM(this.projection, 0, 42.0f, this.surfaceWidth / this.surfaceHeight, 0.1f, 10.0f);
        Matrix.setLookAtM(this.view, 0, 0.0f, 0.0f, 3.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f);
        Matrix.multiplyMM(this.viewProjection, 0, this.projection, 0, this.view, 0);
    }

    private final void updateMatrices() {
        Matrix.setIdentityM(this.model, 0);
        Matrix.rotateM(this.model, 0, this.pitchDegrees, 1.0f, 0.0f, 0.0f);
        Matrix.rotateM(this.model, 0, this.yawDegrees, 0.0f, 1.0f, 0.0f);
        float f = this.meshScale * this.zoom;
        Matrix.scaleM(this.model, 0, f, f, f);
        float[] fArr = this.model;
        float[] fArr2 = this.meshCenter;
        Matrix.translateM(fArr, 0, -fArr2[0], -fArr2[1], -fArr2[2]);
        Matrix.multiplyMM(this.mvp, 0, this.viewProjection, 0, this.model, 0);
    }

    private final void calculateBounds(float[] vertices) {
        float[] fArr = new float[3];
        fArr[0] = Float.POSITIVE_INFINITY;
        fArr[1] = Float.POSITIVE_INFINITY;
        fArr[2] = Float.POSITIVE_INFINITY;
        float[] fArr2 = new float[3];
        fArr2[0] = Float.NEGATIVE_INFINITY;
        fArr2[1] = Float.NEGATIVE_INFINITY;
        fArr2[2] = Float.NEGATIVE_INFINITY;
        for (int i = 0; i < vertices.length; i += 3) {
            for (int i2 = 0; i2 < 3; i2++) {
                int i3 = i + i2;
                fArr[i2] = Math.min(fArr[i2], vertices[i3]);
                fArr2[i2] = Math.max(fArr2[i2], vertices[i3]);
            }
        }
        for (int i4 = 0; i4 < 3; i4++) {
            this.meshCenter[i4] = (fArr[i4] + fArr2[i4]) * 0.5f;
        }
        this.meshScale = 1.65f / RangesKt.coerceAtLeast(Math.max(fArr2[0] - fArr[0], Math.max(fArr2[1] - fArr[1], fArr2[2] - fArr[2])), 0.001f);
    }

    private final void uploadIndices(int buffer, int[] values) {
        IntBuffer asIntBuffer = ByteBuffer.allocateDirect(values.length * 4).order(ByteOrder.nativeOrder()).asIntBuffer();
        asIntBuffer.put(values).position(0);
        GLES30.glBindBuffer(34963, buffer);
        GLES30.glBufferData(34963, values.length * 4, asIntBuffer, 35044);
    }

    private final FloatBuffer allocateFloatBuffer(int count) {
        FloatBuffer asFloatBuffer = ByteBuffer.allocateDirect(count * 4).order(ByteOrder.nativeOrder()).asFloatBuffer();
        Intrinsics.checkNotNullExpressionValue(asFloatBuffer, "asFloatBuffer(...)");
        return asFloatBuffer;
    }

    private final void uploadFloats(int buffer, FloatBuffer upload, float[] values) {
        upload.clear();
        upload.put(values);
        upload.position(0);
        GLES30.glBindBuffer(34962, buffer);
        GLES30.glBufferSubData(34962, 0, values.length * 4, upload);
    }

    private final int createProgram(String vertexSource, String fragmentSource) {
        int compileShader = compileShader(35633, vertexSource);
        int compileShader2 = compileShader(35632, fragmentSource);
        int glCreateProgram = GLES30.glCreateProgram();
        GLES30.glAttachShader(glCreateProgram, compileShader);
        GLES30.glAttachShader(glCreateProgram, compileShader2);
        GLES30.glBindAttribLocation(glCreateProgram, 0, "aPosition");
        GLES30.glBindAttribLocation(glCreateProgram, 1, "aNormal");
        GLES30.glLinkProgram(glCreateProgram);
        int[] iArr = new int[1];
        GLES30.glGetProgramiv(glCreateProgram, 35714, iArr, 0);
        GLES30.glDeleteShader(compileShader);
        GLES30.glDeleteShader(compileShader2);
        if (iArr[0] != 0) {
            return glCreateProgram;
        }
        String glGetProgramInfoLog = GLES30.glGetProgramInfoLog(glCreateProgram);
        GLES30.glDeleteProgram(glCreateProgram);
        throw new IllegalStateException("shader link failed: " + glGetProgramInfoLog);
    }

    private final int compileShader(int type, String source) {
        int glCreateShader = GLES30.glCreateShader(type);
        GLES30.glShaderSource(glCreateShader, source);
        GLES30.glCompileShader(glCreateShader);
        int[] iArr = new int[1];
        GLES30.glGetShaderiv(glCreateShader, 35713, iArr, 0);
        if (iArr[0] != 0) {
            return glCreateShader;
        }
        String glGetShaderInfoLog = GLES30.glGetShaderInfoLog(glCreateShader);
        GLES30.glDeleteShader(glCreateShader);
        throw new IllegalStateException("shader compile failed: " + glGetShaderInfoLog);
    }

    private final void fail(String message) {
        if (this.failureReported) {
            return;
        }
        this.failureReported = true;
        this.reportFailure.invoke(message);
    }
}
