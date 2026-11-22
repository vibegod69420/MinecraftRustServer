package mc.oracle;

import java.io.BufferedWriter;
import java.io.IOException;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicLong;

/**
 * Runtime hooks invoked by bytecode instrumented via {@link OracleAgent}.
 */
public final class OracleRuntime {
    public static final boolean DETERMINISTIC =
        Boolean.parseBoolean(System.getProperty("mc.oracle.deterministic", "false"));
    public static final boolean START_FROZEN =
        Boolean.parseBoolean(System.getProperty("mc.oracle.startFrozen", "false"));

    private static final long INITIAL_SEED =
        Long.getLong("mc.oracle.seed", 0x9E3779B97F4A7C15L);

    private static final AtomicLong UNIQUE_SEED_SEQ = new AtomicLong(0L);
    private static final AtomicLong UUID_SEQ = new AtomicLong(0L);
    private static final ThreadLocal<long[]> THREAD_STREAM_SEQ = ThreadLocal.withInitial(() -> {
        String tname = Thread.currentThread().getName();
        long tag;
        if ("main".equals(tname)) {
            tag = 0L;
        } else if ("Server thread".equals(tname)) {
            tag = 1L;
        } else if ("Worker-Main-1".equals(tname)) {
            tag = 2L;
        } else {
            long h = 0x811c9dc5L;
            for (int i = 0; i < tname.length(); i++) {
                h ^= tname.charAt(i);
                h = (h * 0x01000193L) & 0xFFFFFFL;
            }
            tag = 16L + (h & 0xFFFFFFL);
        }
        return new long[]{tag << 40, 0L};
    });
    private static final AtomicLong TRACE_SEQ = new AtomicLong(0L);

    private static final String TRACE_PATH = System.getProperty("mc.oracle.rng_trace", "");
    public static final boolean TRACE_ENABLED = !TRACE_PATH.isEmpty();
    private static BufferedWriter traceWriter;
    private static final Object TRACE_LOCK = new Object();

    static {
        if (TRACE_ENABLED) {
            try {
                Path p = Path.of(TRACE_PATH);
                if (p.getParent() != null) {
                    Files.createDirectories(p.getParent());
                }
                traceWriter = Files.newBufferedWriter(p, StandardCharsets.UTF_8);
                Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                    synchronized (TRACE_LOCK) {
                        if (traceWriter != null) {
                            try {
                                traceWriter.flush();
                                traceWriter.close();
                            } catch (IOException ignored) {
                            }
                            traceWriter = null;
                        }
                    }
                }, "oracle-trace-flush"));
            } catch (IOException e) {
                throw new RuntimeException("Failed to open rng trace file: " + TRACE_PATH, e);
            }
        }
    }

    private OracleRuntime() {}

    private static long stafford13(long z) {
        z = (z ^ (z >>> 30)) * -4658895280553007687L;
        z = (z ^ (z >>> 27)) * -7723592293110705685L;
        return z ^ (z >>> 31);
    }

    public static long nextUniqueSeed() {
        long idx = UNIQUE_SEED_SEQ.incrementAndGet();
        return stafford13(INITIAL_SEED ^ (idx * 0x9E3779B97F4A7C15L));
    }

    public static UUID nextDeterministicUuid() {
        long idx = UUID_SEQ.incrementAndGet();
        long msb = stafford13(INITIAL_SEED ^ (idx * 0xBF58476D1CE4E5B9L));
        long lsb = stafford13(INITIAL_SEED ^ (idx * 0x94D049BB133111EBL));
        msb = (msb & ~0xF000L) | 0x4000L;
        lsb = (lsb & 0x3FFFFFFFFFFFFFFFL) | 0x8000000000000000L;
        return new UUID(msb, lsb);
    }

    public static double nextDeterministicMathRandom() {
        long bits = nextUniqueSeed() >>> 11;
        return bits * (1.0 / (1L << 53));
    }

    public static long nextStreamId() {
        long[] state = THREAD_STREAM_SEQ.get();
        state[1] += 1L;
        return state[0] | state[1];
    }

    public static void logRng(long streamId, String impl, String method, long arg, long result) {
        if (!TRACE_ENABLED) {
            return;
        }
        long seq = TRACE_SEQ.incrementAndGet();
        synchronized (TRACE_LOCK) {
            if (traceWriter != null) {
                try {
                    traceWriter.write(seq + "\t" + streamId + "\t" + impl + "\t" + method + "\t" + arg + "\t" + result + "\n");
                    if ((seq & 4095L) == 0L) {
                        traceWriter.flush();
                    }
                } catch (IOException ignored) {
                }
            }
        }
    }

    public static int logRngInt(int ret, long streamId, String impl, String method, int arg) {
        logRng(streamId, impl, method, arg, ret);
        return ret;
    }

    public static long logRngLong(long ret, long streamId, String impl, String method, long arg) {
        logRng(streamId, impl, method, arg, ret);
        return ret;
    }

    public static void flushTrace() {
        synchronized (TRACE_LOCK) {
            if (traceWriter != null) {
                try {
                    traceWriter.flush();
                } catch (IOException ignored) {
                }
            }
        }
    }

    public static void maybeFreezeServer(Object minecraftServer) {
        if (!START_FROZEN || minecraftServer == null) {
            return;
        }
        try {
            Method tickRateManager = minecraftServer.getClass().getMethod("tickRateManager");
            Object mgr = tickRateManager.invoke(minecraftServer);
            if (mgr != null) {
                Method setFrozen = mgr.getClass().getMethod("setFrozen", boolean.class);
                setFrozen.invoke(mgr, true);
                System.err.println("[oracle-agent] Server tickRateManager frozen prior to tick 0");
            }
        } catch (Throwable t) {
            System.err.println("[oracle-agent] Failed to freeze server at init: " + t);
        }
    }
}
