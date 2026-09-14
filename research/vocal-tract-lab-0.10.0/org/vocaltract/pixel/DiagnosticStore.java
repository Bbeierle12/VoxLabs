package org.vocaltract.pixel;

import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.Context;
import android.content.pm.PackageInfo;
import android.net.Uri;
import android.os.Build;
import android.os.Environment;
import android.provider.MediaStore;
import java.io.BufferedReader;
import java.io.File;
import java.io.FileFilter;
import java.io.FileInputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.io.Reader;
import java.util.Comparator;
import java.util.Iterator;
import java.util.List;
import java.util.UUID;
import kotlin.Metadata;
import kotlin.Result;
import kotlin.ResultKt;
import kotlin.Unit;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.comparisons.ComparisonsKt;
import kotlin.io.CloseableKt;
import kotlin.io.ConstantsKt;
import kotlin.io.FilesKt;
import kotlin.io.TextStreamsKt;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;
import kotlin.sequences.SequencesKt;
import kotlin.text.Charsets;
import kotlin.text.Regex;
import kotlin.text.StringsKt;
import org.json.JSONArray;
import org.json.JSONObject;

/* compiled from: DiagnosticStore.kt */
@Metadata(d1 = {"\u0000n\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0005\n\u0002\u0018\u0002\n\u0002\b\u0004\n\u0002\u0010\u000b\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0010 \n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0005\u0018\u0000 -2\u00020\u0001:\u0001-B\u0017\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005¢\u0006\u0004\b\u0006\u0010\u0007J\u000e\u0010\u000f\u001a\u00020\u00102\u0006\u0010\u0011\u001a\u00020\u0012J\u000e\u0010\u0013\u001a\u00020\u00142\u0006\u0010\u0015\u001a\u00020\u0016J\b\u0010\u0017\u001a\u0004\u0018\u00010\u0012J\u0016\u0010\u0018\u001a\b\u0012\u0004\u0012\u00020\u00120\u00192\b\b\u0002\u0010\u001a\u001a\u00020\u001bJ\u0016\u0010\u001c\u001a\b\u0012\u0004\u0012\u00020\u001d0\u00192\b\b\u0002\u0010\u001a\u001a\u00020\u001bJ\u0006\u0010\u001e\u001a\u00020\u0014J<\u0010\u001f\u001a\u00020 2\u0006\u0010!\u001a\u00020\"2\u0006\u0010#\u001a\u00020$2\f\u0010%\u001a\b\u0012\u0004\u0012\u00020&0\u00192\n\b\u0002\u0010'\u001a\u0004\u0018\u00010(2\n\b\u0002\u0010)\u001a\u0004\u0018\u00010\u0005J\b\u0010*\u001a\u00020\u001dH\u0002J\b\u0010+\u001a\u00020\u001dH\u0002J\b\u0010,\u001a\u00020\u0014H\u0002R\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\b\u0010\tR\u000e\u0010\n\u001a\u00020\u000bX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\f\u001a\u00020\u000bX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\r\u001a\u00020\u000bX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000e\u001a\u00020\u000bX\u0082\u0004¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticStore;", "", "context", "Landroid/content/Context;", "sessionId", "", "<init>", "(Landroid/content/Context;Ljava/lang/String;)V", "getSessionId", "()Ljava/lang/String;", "directory", "Ljava/io/File;", "sessionFile", "refinementFile", "pendingCrashFile", "append", "", "event", "Lorg/vocaltract/pixel/DiagnosticEvent;", "appendRefinement", "", "record", "Lorg/vocaltract/pixel/RefinementRecord;", "consumePendingCrash", "recentEvents", "", "limit", "", "refinements", "Lorg/json/JSONObject;", "clear", "exportBundle", "Lorg/vocaltract/pixel/DiagnosticExport;", "metrics", "Lorg/vocaltract/pixel/DiagnosticMetrics;", "assessment", "Lorg/vocaltract/pixel/DiagnosticAssessment;", "selfTests", "Lorg/vocaltract/pixel/SelfTestResult;", "calibration", "Lorg/vocaltract/pixel/SingerCalibrationReport;", "sharedModelSnapshot", "appJson", "deviceJson", "prune", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class DiagnosticStore {

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    private static final int MAX_EXPORTED_EVENTS = 1500;
    private static final int MAX_EXPORTED_REFINEMENTS = 500;
    private static final long MAX_REFINEMENT_BYTES = 4194304;
    private static final long MAX_SESSION_BYTES = 4194304;
    private static final int MAX_SESSION_FILES = 20;
    private final Context context;
    private final File directory;
    private final File pendingCrashFile;
    private final File refinementFile;
    private final File sessionFile;
    private final String sessionId;

    public DiagnosticStore(Context context, String sessionId) {
        Intrinsics.checkNotNullParameter(context, "context");
        Intrinsics.checkNotNullParameter(sessionId, "sessionId");
        this.context = context;
        this.sessionId = sessionId;
        File file = new File(context.getFilesDir(), "diagnostics");
        this.directory = file;
        this.sessionFile = new File(file, "session-" + sessionId + ".jsonl");
        this.refinementFile = new File(file, "refinements.jsonl");
        this.pendingCrashFile = new File(file, "pending-crash.json");
        if (!new Regex("[A-Za-z0-9_-]{8,80}").matches(sessionId)) {
            throw new IllegalArgumentException("Invalid diagnostics session ID".toString());
        }
        file.mkdirs();
        prune();
    }

    public final String getSessionId() {
        return this.sessionId;
    }

    public final synchronized boolean append(DiagnosticEvent event) {
        JSONObject json;
        Intrinsics.checkNotNullParameter(event, "event");
        if (this.sessionFile.length() >= 4194304) {
            return false;
        }
        this.directory.mkdirs();
        json = DiagnosticStoreKt.toJson(event);
        String jSONObject = json.toString();
        Intrinsics.checkNotNullExpressionValue(jSONObject, "toString(...)");
        FilesKt.appendText(this.sessionFile, jSONObject + "\n", Charsets.UTF_8);
        if (Intrinsics.areEqual(event.getCategory(), "crash")) {
            FilesKt.writeText(this.pendingCrashFile, jSONObject, Charsets.UTF_8);
        }
        return true;
    }

    public final synchronized void appendRefinement(RefinementRecord record) {
        JSONObject json;
        Intrinsics.checkNotNullParameter(record, "record");
        this.directory.mkdirs();
        if (this.refinementFile.length() >= 4194304) {
            throw new IllegalStateException("Refinement store is full; export and clear local diagnostics before adding more".toString());
        }
        File file = this.refinementFile;
        json = DiagnosticStoreKt.toJson(record);
        FilesKt.appendText(file, json + "\n", Charsets.UTF_8);
    }

    public final synchronized DiagnosticEvent consumePendingCrash() {
        Object m4constructorimpl;
        DiagnosticEvent eventFromJson;
        Object obj = null;
        if (!this.pendingCrashFile.isFile()) {
            return null;
        }
        try {
            Result.Companion companion = Result.INSTANCE;
            DiagnosticStore diagnosticStore = this;
            eventFromJson = DiagnosticStoreKt.eventFromJson(new JSONObject(FilesKt.readText(this.pendingCrashFile, Charsets.UTF_8)));
            m4constructorimpl = Result.m4constructorimpl(eventFromJson);
        } catch (Throwable th) {
            Result.Companion companion2 = Result.INSTANCE;
            m4constructorimpl = Result.m4constructorimpl(ResultKt.createFailure(th));
        }
        if (!Result.m10isFailureimpl(m4constructorimpl)) {
            obj = m4constructorimpl;
        }
        DiagnosticEvent diagnosticEvent = (DiagnosticEvent) obj;
        if (diagnosticEvent != null) {
            this.pendingCrashFile.delete();
        }
        return diagnosticEvent;
    }

    public static /* synthetic */ List recentEvents$default(DiagnosticStore diagnosticStore, int i, int i2, Object obj) {
        if ((i2 & 1) != 0) {
            i = 80;
        }
        return diagnosticStore.recentEvents(i);
    }

    /* JADX DEBUG: Another duplicated slice has different insns count: {[]}, finally: {[THROW, INVOKE, MOVE_EXCEPTION, THROW, MOVE_EXCEPTION] complete} */
    /* JADX DEBUG: Finally have unexpected throw blocks count: 2, expect 1 */
    public final synchronized List<DiagnosticEvent> recentEvents(int limit) {
        if (!this.sessionFile.isFile()) {
            return CollectionsKt.emptyList();
        }
        Reader inputStreamReader = new InputStreamReader(new FileInputStream(this.sessionFile), Charsets.UTF_8);
        BufferedReader bufferedReader = inputStreamReader instanceof BufferedReader ? (BufferedReader) inputStreamReader : new BufferedReader(inputStreamReader, ConstantsKt.DEFAULT_BUFFER_SIZE);
        try {
            List<DiagnosticEvent> takeLast = CollectionsKt.takeLast(SequencesKt.toList(SequencesKt.mapNotNull(TextStreamsKt.lineSequence(bufferedReader), new Function1() { // from class: org.vocaltract.pixel.DiagnosticStore$$ExternalSyntheticLambda1
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function1
                public final Object invoke(Object obj) {
                    DiagnosticEvent recentEvents$lambda$5$lambda$4;
                    recentEvents$lambda$5$lambda$4 = DiagnosticStore.recentEvents$lambda$5$lambda$4(DiagnosticStore.this, (String) obj);
                    return recentEvents$lambda$5$lambda$4;
                }
            })), RangesKt.coerceIn(limit, 1, MAX_EXPORTED_EVENTS));
            CloseableKt.closeFinally(bufferedReader, null);
            return takeLast;
        } finally {
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final DiagnosticEvent recentEvents$lambda$5$lambda$4(DiagnosticStore diagnosticStore, String line) {
        Object m4constructorimpl;
        DiagnosticEvent eventFromJson;
        Intrinsics.checkNotNullParameter(line, "line");
        try {
            Result.Companion companion = Result.INSTANCE;
            eventFromJson = DiagnosticStoreKt.eventFromJson(new JSONObject(line));
            m4constructorimpl = Result.m4constructorimpl(eventFromJson);
        } catch (Throwable th) {
            Result.Companion companion2 = Result.INSTANCE;
            m4constructorimpl = Result.m4constructorimpl(ResultKt.createFailure(th));
        }
        if (Result.m10isFailureimpl(m4constructorimpl)) {
            m4constructorimpl = null;
        }
        return (DiagnosticEvent) m4constructorimpl;
    }

    public static /* synthetic */ List refinements$default(DiagnosticStore diagnosticStore, int i, int i2, Object obj) {
        if ((i2 & 1) != 0) {
            i = 200;
        }
        return diagnosticStore.refinements(i);
    }

    /* JADX DEBUG: Another duplicated slice has different insns count: {[]}, finally: {[THROW, INVOKE, MOVE_EXCEPTION, THROW, MOVE_EXCEPTION] complete} */
    /* JADX DEBUG: Finally have unexpected throw blocks count: 2, expect 1 */
    public final synchronized List<JSONObject> refinements(int limit) {
        if (!this.refinementFile.isFile()) {
            return CollectionsKt.emptyList();
        }
        Reader inputStreamReader = new InputStreamReader(new FileInputStream(this.refinementFile), Charsets.UTF_8);
        BufferedReader bufferedReader = inputStreamReader instanceof BufferedReader ? (BufferedReader) inputStreamReader : new BufferedReader(inputStreamReader, ConstantsKt.DEFAULT_BUFFER_SIZE);
        try {
            List<JSONObject> takeLast = CollectionsKt.takeLast(SequencesKt.toList(SequencesKt.mapNotNull(TextStreamsKt.lineSequence(bufferedReader), new Function1() { // from class: org.vocaltract.pixel.DiagnosticStore$$ExternalSyntheticLambda2
                /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
                @Override // kotlin.jvm.functions.Function1
                public final Object invoke(Object obj) {
                    JSONObject refinements$lambda$8$lambda$7;
                    refinements$lambda$8$lambda$7 = DiagnosticStore.refinements$lambda$8$lambda$7(DiagnosticStore.this, (String) obj);
                    return refinements$lambda$8$lambda$7;
                }
            })), RangesKt.coerceIn(limit, 1, MAX_EXPORTED_REFINEMENTS));
            CloseableKt.closeFinally(bufferedReader, null);
            return takeLast;
        } finally {
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final JSONObject refinements$lambda$8$lambda$7(DiagnosticStore diagnosticStore, String line) {
        Object m4constructorimpl;
        Intrinsics.checkNotNullParameter(line, "line");
        try {
            Result.Companion companion = Result.INSTANCE;
            m4constructorimpl = Result.m4constructorimpl(new JSONObject(line));
        } catch (Throwable th) {
            Result.Companion companion2 = Result.INSTANCE;
            m4constructorimpl = Result.m4constructorimpl(ResultKt.createFailure(th));
        }
        if (Result.m10isFailureimpl(m4constructorimpl)) {
            m4constructorimpl = null;
        }
        return (JSONObject) m4constructorimpl;
    }

    public final synchronized void clear() {
        File[] listFiles = this.directory.listFiles();
        if (listFiles != null) {
            for (File file : listFiles) {
                if (file.isFile() && Intrinsics.areEqual(file.getParentFile(), this.directory)) {
                    file.delete();
                }
            }
        }
    }

    /* JADX DEBUG: Another duplicated slice has different insns count: {[]}, finally: {[THROW, INVOKE, MOVE_EXCEPTION, THROW, MOVE_EXCEPTION] complete} */
    /* JADX DEBUG: Finally have unexpected throw blocks count: 2, expect 1 */
    /* JADX WARN: Code restructure failed: missing block: B:8:0x0095, code lost:
    
        r6 = org.vocaltract.pixel.DiagnosticStoreKt.toJson(r9);
     */
    /*
        Code decompiled incorrectly, please refer to instructions dump.
    */
    public final DiagnosticExport exportBundle(DiagnosticMetrics metrics, DiagnosticAssessment assessment, List<SelfTestResult> selfTests, SingerCalibrationReport calibration, String sharedModelSnapshot) {
        JSONObject json;
        JSONObject json2;
        Object obj;
        JSONObject json3;
        JSONObject json4;
        Intrinsics.checkNotNullParameter(metrics, "metrics");
        Intrinsics.checkNotNullParameter(assessment, "assessment");
        Intrinsics.checkNotNullParameter(selfTests, "selfTests");
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("schema_version", "vocaltract3d.diagnostic-bundle/1.1");
        jSONObject.put("generated_at_epoch_ms", System.currentTimeMillis());
        jSONObject.put("session_id", this.sessionId);
        JSONObject jSONObject2 = new JSONObject();
        jSONObject2.put("raw_audio_included", false);
        jSONObject2.put("internet_upload_performed", false);
        jSONObject2.put("contents", "derived metrics, app events, explicit user corrections, and self-test results");
        Unit unit = Unit.INSTANCE;
        jSONObject.put("privacy", jSONObject2);
        jSONObject.put("app", appJson());
        jSONObject.put("device", deviceJson());
        json = DiagnosticStoreKt.toJson(metrics);
        jSONObject.put("latest_metrics", json);
        json2 = DiagnosticStoreKt.toJson(assessment);
        jSONObject.put("assistant_assessment", json2);
        JSONArray jSONArray = new JSONArray();
        Iterator<T> it = selfTests.iterator();
        while (it.hasNext()) {
            json4 = DiagnosticStoreKt.toJson((SelfTestResult) it.next());
            jSONArray.put(json4);
        }
        Unit unit2 = Unit.INSTANCE;
        jSONObject.put("self_tests", jSONArray);
        if (calibration == null || obj == null) {
            obj = JSONObject.NULL;
        }
        jSONObject.put("singer_calibration", obj);
        jSONObject.put("shared_model", sharedModelSnapshot != null ? new JSONObject(sharedModelSnapshot) : JSONObject.NULL);
        JSONArray jSONArray2 = new JSONArray();
        Iterator<T> it2 = recentEvents(MAX_EXPORTED_EVENTS).iterator();
        while (it2.hasNext()) {
            json3 = DiagnosticStoreKt.toJson((DiagnosticEvent) it2.next());
            jSONArray2.put(json3);
        }
        Unit unit3 = Unit.INSTANCE;
        jSONObject.put("events", jSONArray2);
        JSONArray jSONArray3 = new JSONArray();
        Iterator<T> it3 = refinements(MAX_EXPORTED_REFINEMENTS).iterator();
        while (it3.hasNext()) {
            jSONArray3.put(it3.next());
        }
        Unit unit4 = Unit.INSTANCE;
        jSONObject.put("refinements", jSONArray3);
        String jSONObject3 = jSONObject.toString(2);
        Intrinsics.checkNotNullExpressionValue(jSONObject3, "toString(...)");
        byte[] bytes = jSONObject3.getBytes(Charsets.UTF_8);
        Intrinsics.checkNotNullExpressionValue(bytes, "getBytes(...)");
        String str = "vocal-tract-diagnostics-" + this.sessionId + ".json";
        ContentValues contentValues = new ContentValues();
        contentValues.put("_display_name", str);
        contentValues.put("mime_type", "application/json");
        contentValues.put("relative_path", Environment.DIRECTORY_DOWNLOADS + "/VocalTractLab");
        contentValues.put("is_pending", (Integer) 1);
        ContentResolver contentResolver = this.context.getContentResolver();
        Uri insert = contentResolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues);
        if (insert == null) {
            throw new IllegalArgumentException("Android could not create the diagnostics download".toString());
        }
        try {
            OutputStream openOutputStream = contentResolver.openOutputStream(insert, "w");
            if (openOutputStream == null) {
                throw new IllegalArgumentException("Required value was null.".toString());
            }
            OutputStream outputStream = openOutputStream;
            try {
                outputStream.write(bytes);
                Unit unit5 = Unit.INSTANCE;
                CloseableKt.closeFinally(outputStream, null);
                contentValues.clear();
                contentValues.put("is_pending", (Integer) 0);
                contentResolver.update(insert, contentValues, null, null);
                String uri = insert.toString();
                Intrinsics.checkNotNullExpressionValue(uri, "toString(...)");
                return new DiagnosticExport(str, uri, bytes.length);
            } finally {
            }
        } catch (Throwable th) {
            contentResolver.delete(insert, null, null);
            throw th;
        }
    }

    private final JSONObject appJson() {
        PackageInfo packageInfo = this.context.getPackageManager().getPackageInfo(this.context.getPackageName(), 0);
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("package", this.context.getPackageName());
        String str = packageInfo.versionName;
        if (str == null) {
            str = "unknown";
        }
        jSONObject.put("version_name", str);
        jSONObject.put("version_code", packageInfo.getLongVersionCode());
        return jSONObject;
    }

    private final JSONObject deviceJson() {
        JSONObject jSONObject = new JSONObject();
        jSONObject.put("manufacturer", Build.MANUFACTURER);
        jSONObject.put("model", Build.MODEL);
        jSONObject.put("android_api", Build.VERSION.SDK_INT);
        jSONObject.put("android_release", Build.VERSION.RELEASE);
        JSONArray jSONArray = new JSONArray();
        String[] SUPPORTED_ABIS = Build.SUPPORTED_ABIS;
        Intrinsics.checkNotNullExpressionValue(SUPPORTED_ABIS, "SUPPORTED_ABIS");
        for (String str : SUPPORTED_ABIS) {
            jSONArray.put(str);
        }
        Unit unit = Unit.INSTANCE;
        jSONObject.put("supported_abis", jSONArray);
        return jSONObject;
    }

    private final void prune() {
        File[] listFiles = this.directory.listFiles(new FileFilter() { // from class: org.vocaltract.pixel.DiagnosticStore$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // java.io.FileFilter
            public final boolean accept(File file) {
                boolean prune$lambda$25;
                prune$lambda$25 = DiagnosticStore.prune$lambda$25(file);
                return prune$lambda$25;
            }
        });
        List sortedWith = listFiles != null ? ArraysKt.sortedWith(listFiles, new Comparator() { // from class: org.vocaltract.pixel.DiagnosticStore$prune$$inlined$sortedByDescending$1
            /* JADX DEBUG: Multi-variable search result rejected for r3v0, resolved type: T */
            /* JADX DEBUG: Multi-variable search result rejected for r4v0, resolved type: T */
            /* JADX WARN: Multi-variable type inference failed */
            @Override // java.util.Comparator
            public final int compare(T t, T t2) {
                return ComparisonsKt.compareValues(Long.valueOf(((File) t2).lastModified()), Long.valueOf(((File) t).lastModified()));
            }
        }) : null;
        if (sortedWith == null) {
            sortedWith = CollectionsKt.emptyList();
        }
        Iterator it = CollectionsKt.drop(sortedWith, MAX_SESSION_FILES).iterator();
        while (it.hasNext()) {
            ((File) it.next()).delete();
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final boolean prune$lambda$25(File file) {
        if (!file.isFile()) {
            return false;
        }
        String name = file.getName();
        Intrinsics.checkNotNullExpressionValue(name, "getName(...)");
        if (!StringsKt.startsWith$default(name, "session-", false, 2, (Object) null)) {
            return false;
        }
        String name2 = file.getName();
        Intrinsics.checkNotNullExpressionValue(name2, "getName(...)");
        return StringsKt.endsWith$default(name2, ".jsonl", false, 2, (Object) null);
    }

    /* compiled from: DiagnosticStore.kt */
    @Metadata(d1 = {"\u0000\"\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\t\n\u0002\b\u0002\n\u0002\u0010\b\n\u0002\b\u0003\n\u0002\u0010\u000e\n\u0000\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u0010\u0010\u000b\u001a\u00020\f2\b\b\u0002\u0010\r\u001a\u00020\u0005R\u000e\u0010\u0004\u001a\u00020\u0005X\u0082T¢\u0006\u0002\n\u0000R\u000e\u0010\u0006\u001a\u00020\u0005X\u0082T¢\u0006\u0002\n\u0000R\u000e\u0010\u0007\u001a\u00020\bX\u0082T¢\u0006\u0002\n\u0000R\u000e\u0010\t\u001a\u00020\bX\u0082T¢\u0006\u0002\n\u0000R\u000e\u0010\n\u001a\u00020\bX\u0082T¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/DiagnosticStore$Companion;", "", "<init>", "()V", "MAX_SESSION_BYTES", "", "MAX_REFINEMENT_BYTES", "MAX_SESSION_FILES", "", "MAX_EXPORTED_EVENTS", "MAX_EXPORTED_REFINEMENTS", "newSessionId", "", "nowMillis"}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        public static /* synthetic */ String newSessionId$default(Companion companion, long j, int i, Object obj) {
            if ((i & 1) != 0) {
                j = System.currentTimeMillis();
            }
            return companion.newSessionId(j);
        }

        public final String newSessionId(long nowMillis) {
            String uuid = UUID.randomUUID().toString();
            Intrinsics.checkNotNullExpressionValue(uuid, "toString(...)");
            return nowMillis + "-" + StringsKt.take(uuid, 8);
        }
    }
}
