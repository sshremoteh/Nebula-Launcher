package dev.horizon.android;

import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;

/** SAF result callback only. The native activity, UI and file operations stay Rust. */
public final class HorizonActivity extends android.app.NativeActivity {
    static { System.loadLibrary("horizon_android"); }

    private static final int REQUEST_DOCUMENT = 4101;
    private native void nativeDocumentResult(int request, int result, String uri);
    private native void nativeImportUri(String uri);

    private static HorizonActivity instance;

    public static HorizonActivity getInstance() {
        return instance;
    }

    static final class ImportTask implements Runnable {
        final HorizonActivity activity;
        final String uri;
        ImportTask(HorizonActivity activity, String uri) {
            this.activity = activity;
            this.uri = uri;
        }
        @Override
        public void run() {
            if (activity != null) {
                activity.nativeImportUri(uri);
            }
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        instance = this;
        super.onCreate(savedInstanceState);
        handleExternalIntent(getIntent());
    }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        handleExternalIntent(intent);
    }

    @Override
    protected void onDestroy() {
        if (instance == this) instance = null;
        super.onDestroy();
    }

    private Uri extractUri(Intent data) {
        if (data == null) return null;
        Uri uri = data.getData();
        if (uri == null && data.getClipData() != null && data.getClipData().getItemCount() > 0) {
            uri = data.getClipData().getItemAt(0).getUri();
        }
        if (uri == null) {
            try {
                uri = data.getParcelableExtra(Intent.EXTRA_STREAM);
            } catch (Exception ignored) {}
        }
        return uri;
    }

    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != REQUEST_DOCUMENT) return;
        Uri uri = extractUri(data);
        if (uri != null) {
            try {
                getContentResolver().takePersistableUriPermission(uri,
                        Intent.FLAG_GRANT_READ_URI_PERMISSION);
            } catch (Exception ignored) {}
        }
        nativeDocumentResult(request, result, uri == null ? null : uri.toString());
    }

    private void handleExternalIntent(Intent intent) {
        if (intent == null) return;
        String action = intent.getAction();
        if (Intent.ACTION_VIEW.equals(action) || Intent.ACTION_SEND.equals(action)) {
            final Uri uri = extractUri(intent);
            if (uri != null) {
                try {
                    getContentResolver().takePersistableUriPermission(uri,
                            Intent.FLAG_GRANT_READ_URI_PERMISSION);
                } catch (Exception ignored) {}
                final String uriStr = uri.toString();
                getWindow().getDecorView().post(new ImportTask(this, uriStr));
            }
        }
    }

    public void launchDocumentPicker(int requestCode) {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType("*/*");
        intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION);
        try {
            startActivityForResult(intent, requestCode);
        } catch (Exception e) {
            try {
                Intent fallback = new Intent(Intent.ACTION_GET_CONTENT);
                fallback.addCategory(Intent.CATEGORY_OPENABLE);
                fallback.setType("*/*");
                fallback.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
                startActivityForResult(Intent.createChooser(fallback, "Import mod / file"), requestCode);
            } catch (Exception err) {
                nativeDocumentResult(requestCode, 0, null);
            }
        }
    }
}
