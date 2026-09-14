package org.vocaltract.pixel;

import java.net.URI;
import java.util.Map;
import kotlin.Metadata;
import kotlin.Pair;
import kotlin.TuplesKt;
import kotlin.collections.MapsKt;
import kotlin.jvm.internal.Intrinsics;
import kotlin.text.StringsKt;

/* compiled from: LabAudioContract.kt */
@Metadata(d1 = {"\u0000\u001e\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010$\n\u0000\n\u0002\u0018\u0002\n\u0000\bÆ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u001c\u0010\b\u001a\u0010\u0012\u0004\u0012\u00020\u0005\u0012\u0004\u0012\u00020\u0005\u0018\u00010\t2\u0006\u0010\n\u001a\u00020\u0005R\u000e\u0010\u0004\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000R\u001a\u0010\u0006\u001a\u000e\u0012\u0004\u0012\u00020\u0005\u0012\u0004\u0012\u00020\u00050\u0007X\u0082\u0004¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/LabAssets;", "", "<init>", "()V", "PAGE", "", "types", "", "resolve", "Lkotlin/Pair;", "url"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class LabAssets {
    public static final String PAGE = "https://appassets.androidplatform.net/anatomy/index.html";
    public static final LabAssets INSTANCE = new LabAssets();
    private static final Map<String, String> types = MapsKt.mapOf(TuplesKt.to("index.html", "text/html"), TuplesKt.to("bench.css", "text/css"), TuplesKt.to("lab-view.js", "application/javascript"), TuplesKt.to("bench.js", "application/javascript"), TuplesKt.to("reference.js", "application/javascript"));

    private LabAssets() {
    }

    public final Pair<String, String> resolve(String url) {
        Intrinsics.checkNotNullParameter(url, "url");
        try {
            URI uri = new URI(url);
            String rawPath = uri.getRawPath();
            String removePrefix = rawPath != null ? StringsKt.removePrefix(rawPath, (CharSequence) "/anatomy/") : null;
            String str = types.get(removePrefix);
            if (!Intrinsics.areEqual(uri.getScheme(), "https") || !Intrinsics.areEqual(uri.getHost(), "appassets.androidplatform.net") || uri.getPort() != -1 || uri.getRawUserInfo() != null || uri.getRawQuery() != null || uri.getRawFragment() != null) {
                return null;
            }
            if (Intrinsics.areEqual(uri.getRawPath(), "/anatomy/" + removePrefix) && str != null) {
                return new Pair<>("anatomy/" + removePrefix, str);
            }
            return null;
        } catch (Exception unused) {
            return null;
        }
    }
}
