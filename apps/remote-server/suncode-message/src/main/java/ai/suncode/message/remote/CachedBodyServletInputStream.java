package ai.suncode.message.remote;

import jakarta.servlet.ReadListener;
import jakarta.servlet.ServletInputStream;

import java.io.ByteArrayInputStream;

public final class CachedBodyServletInputStream extends ServletInputStream {
    private final ByteArrayInputStream input;

    public CachedBodyServletInputStream(byte[] body) {
        input = new ByteArrayInputStream(body);
    }

    @Override
    public int read() {
        return input.read();
    }

    @Override
    public boolean isFinished() {
        return input.available() == 0;
    }

    @Override
    public boolean isReady() {
        return true;
    }

    @Override
    public void setReadListener(ReadListener listener) {
    }
}
