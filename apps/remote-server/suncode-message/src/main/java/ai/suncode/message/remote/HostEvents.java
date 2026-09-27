package ai.suncode.message.remote;

import java.util.ArrayDeque;
import java.util.concurrent.atomic.AtomicLong;

public final class HostEvents {
    private final AtomicLong sequence = new AtomicLong();
    private final ArrayDeque<MobileEvent> replay = new ArrayDeque<>();

    public AtomicLong sequence() {
        return sequence;
    }

    public ArrayDeque<MobileEvent> replay() {
        return replay;
    }
}
