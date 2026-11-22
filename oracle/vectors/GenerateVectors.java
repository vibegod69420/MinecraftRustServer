package mc.oracle;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import it.unimi.dsi.fastutil.doubles.DoubleArrayList;
import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.SectionPos;
import net.minecraft.nbt.ByteArrayTag;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.DoubleTag;
import net.minecraft.nbt.FloatTag;
import net.minecraft.nbt.IntArrayTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.LongArrayTag;
import net.minecraft.nbt.NbtIo;
import net.minecraft.nbt.StringTag;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.util.SimpleBitStorage;
import net.minecraft.world.level.levelgen.LegacyRandomSource;
import net.minecraft.world.level.levelgen.PositionalRandomFactory;
import net.minecraft.world.level.levelgen.RandomSupport;
import net.minecraft.world.level.levelgen.WorldgenRandom;
import net.minecraft.world.level.levelgen.XoroshiroRandomSource;
import net.minecraft.world.level.levelgen.synth.ImprovedNoise;
import net.minecraft.world.level.levelgen.synth.NormalNoise;
import net.minecraft.world.level.levelgen.synth.PerlinNoise;
import net.minecraft.world.level.levelgen.synth.SimplexNoise;

/**
 * Generates Method 1 bit-exact golden vectors from the 1.21.11 Java reference
 * and audits Math.* vs StrictMath.* across 1,000,000 inputs per operation.
 */
public final class GenerateVectors {
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();

    public static void main(String[] args) throws Exception {
        Path outDir = Path.of(args.length > 0 ? args[0] : "reference/vectors");
        boolean auditOnly = args.length > 1 && args[1].equals("--audit-only");
        Files.createDirectories(outDir);

        JsonObject audit = runMathAudit();
        Files.writeString(outDir.resolve("math_audit.json"), GSON.toJson(audit), StandardCharsets.UTF_8);
        System.out.println("[vectors] Wrote math_audit.json");
        if (auditOnly) {
            return;
        }

        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();

        Files.writeString(outDir.resolve("random.json"), GSON.toJson(genRandom()), StandardCharsets.UTF_8);
        System.out.println("[vectors] Wrote random.json");

        Files.writeString(outDir.resolve("mth.json"), GSON.toJson(genMth()), StandardCharsets.UTF_8);
        System.out.println("[vectors] Wrote mth.json");

        Files.writeString(outDir.resolve("noise.json"), GSON.toJson(genNoise()), StandardCharsets.UTF_8);
        System.out.println("[vectors] Wrote noise.json");

        Files.writeString(outDir.resolve("packing.json"), GSON.toJson(genPacking()), StandardCharsets.UTF_8);
        System.out.println("[vectors] Wrote packing.json");

        Files.writeString(outDir.resolve("nbt.json"), GSON.toJson(genNbt()), StandardCharsets.UTF_8);
        System.out.println("[vectors] Wrote nbt.json");
    }

    private static String f32Hex(float v) {
        return String.format("%08x", Float.floatToRawIntBits(v));
    }

    private static String f64Hex(double v) {
        return String.format("%016x", Double.doubleToRawLongBits(v));
    }

    private static JsonObject runMathAudit() {
        JsonObject root = new JsonObject();
        root.addProperty("arch", System.getProperty("os.arch"));
        root.addProperty("os", System.getProperty("os.name"));
        root.addProperty("java_version", System.getProperty("java.version"));
        root.addProperty("java_vm_version", System.getProperty("java.vm.version"));

        int n = 1_000_000;
        long sinDiffs = 0, cosDiffs = 0, powDiffs = 0, expDiffs = 0, logDiffs = 0, sqrtDiffs = 0, atan2Diffs = 0;

        // Deterministic splitmix64 generator so audit inputs are identical across architectures
        long s = 0x123456789abcdef0L;
        for (int i = 0; i < n; i++) {
            s += 0x9e3779b97f4a7c15L;
            long z = (s ^ (s >>> 30)) * 0xbf58476d1ce4e5b9L;
            z = (z ^ (z >>> 27)) * 0x94d049bb133111ebL;
            z ^= (z >>> 31);
            double x = ((z & 0xFFFFFFFFL) / (double) 0x100000000L) * 2000.0 - 1000.0;
            double posX = Math.abs(x) + 1e-12;
            double smallX = x * 0.01;
            double y = (((z >>> 32) & 0xFFFFFFFFL) / (double) 0x100000000L) * 20.0 - 10.0;

            if (Double.doubleToRawLongBits(Math.sin(x)) != Double.doubleToRawLongBits(StrictMath.sin(x))) sinDiffs++;
            if (Double.doubleToRawLongBits(Math.cos(x)) != Double.doubleToRawLongBits(StrictMath.cos(x))) cosDiffs++;
            if (Double.doubleToRawLongBits(Math.pow(posX, y)) != Double.doubleToRawLongBits(StrictMath.pow(posX, y))) powDiffs++;
            if (Double.doubleToRawLongBits(Math.exp(smallX)) != Double.doubleToRawLongBits(StrictMath.exp(smallX))) expDiffs++;
            if (Double.doubleToRawLongBits(Math.log(posX)) != Double.doubleToRawLongBits(StrictMath.log(posX))) logDiffs++;
            if (Double.doubleToRawLongBits(Math.sqrt(posX)) != Double.doubleToRawLongBits(StrictMath.sqrt(posX))) sqrtDiffs++;
            if (Double.doubleToRawLongBits(Math.atan2(y, x)) != Double.doubleToRawLongBits(StrictMath.atan2(y, x))) atan2Diffs++;
        }
        JsonObject diffs = new JsonObject();
        diffs.addProperty("samples", n);
        diffs.addProperty("sin", sinDiffs);
        diffs.addProperty("cos", cosDiffs);
        diffs.addProperty("pow", powDiffs);
        diffs.addProperty("exp", expDiffs);
        diffs.addProperty("log", logDiffs);
        diffs.addProperty("sqrt", sqrtDiffs);
        diffs.addProperty("atan2", atan2Diffs);
        root.add("math_vs_strictmath_diffs", diffs);
        return root;
    }

    private static JsonObject captureStream(RandomSource rng) {
        JsonObject obj = new JsonObject();
        JsonArray ints = new JsonArray();
        for (int i = 0; i < 16; i++) ints.add(rng.nextInt());
        obj.add("next_int", ints);

        int[] bounds = {1, 2, 3, 7, 16, 17, 31, 256, 1000000};
        JsonArray bounded = new JsonArray();
        for (int b : bounds) {
            for (int i = 0; i < 3; i++) {
                JsonArray pair = new JsonArray();
                pair.add(b);
                pair.add(rng.nextInt(b));
                bounded.add(pair);
            }
        }
        obj.add("next_int_bounded", bounded);

        JsonArray longs = new JsonArray();
        for (int i = 0; i < 8; i++) longs.add(Long.toString(rng.nextLong()));
        obj.add("next_long", longs);

        JsonArray bools = new JsonArray();
        for (int i = 0; i < 16; i++) bools.add(rng.nextBoolean());
        obj.add("next_boolean", bools);

        JsonArray floats = new JsonArray();
        for (int i = 0; i < 8; i++) floats.add(f32Hex(rng.nextFloat()));
        obj.add("next_float_bits", floats);

        JsonArray doubles = new JsonArray();
        for (int i = 0; i < 8; i++) doubles.add(f64Hex(rng.nextDouble()));
        obj.add("next_double_bits", doubles);

        JsonArray gauss = new JsonArray();
        for (int i = 0; i < 8; i++) gauss.add(f64Hex(rng.nextGaussian()));
        obj.add("next_gaussian_bits", gauss);

        RandomSource forked = rng.fork();
        JsonArray forkedLongs = new JsonArray();
        for (int i = 0; i < 4; i++) forkedLongs.add(Long.toString(forked.nextLong()));
        obj.add("fork_longs", forkedLongs);

        PositionalRandomFactory pos = rng.forkPositional();
        JsonArray posChecks = new JsonArray();
        int[][] coords = {{0, 0, 0}, {1, -5, 200}, {-12345, 319, 98765}};
        for (int[] c : coords) {
            JsonObject p = new JsonObject();
            p.addProperty("x", c[0]);
            p.addProperty("y", c[1]);
            p.addProperty("z", c[2]);
            RandomSource pr = pos.at(c[0], c[1], c[2]);
            p.addProperty("next_long", Long.toString(pr.nextLong()));
            p.addProperty("next_int", pr.nextInt());
            posChecks.add(p);
        }
        obj.add("pos_at", posChecks);

        JsonArray hashChecks = new JsonArray();
        for (String s : new String[]{"", "octave_-7", "minecraft:overworld", "aquifer"}) {
            JsonObject h = new JsonObject();
            h.addProperty("name", s);
            RandomSource hr = pos.fromHashOf(s);
            h.addProperty("next_long", Long.toString(hr.nextLong()));
            h.addProperty("next_int", hr.nextInt());
            hashChecks.add(h);
        }
        obj.add("pos_hash", hashChecks);
        return obj;
    }

    private static JsonObject genRandom() {
        JsonObject root = new JsonObject();

        long[] seeds = {0L, 1L, -1L, 123456789L, Long.MIN_VALUE, Long.MAX_VALUE, 0x123456789abcdef0L};
        JsonArray stafford = new JsonArray();
        JsonArray upgrades = new JsonArray();
        for (long s : seeds) {
            JsonObject st = new JsonObject();
            st.addProperty("in", Long.toString(s));
            st.addProperty("out", Long.toString(RandomSupport.mixStafford13(s)));
            stafford.add(st);

            RandomSupport.Seed128bit u = RandomSupport.upgradeSeedTo128bit(s);
            JsonObject up = new JsonObject();
            up.addProperty("in", Long.toString(s));
            up.addProperty("lo", Long.toString(u.seedLo()));
            up.addProperty("hi", Long.toString(u.seedHi()));
            upgrades.add(up);
        }
        root.add("stafford13", stafford);
        root.add("upgrade_128", upgrades);

        JsonArray hashSeeds = new JsonArray();
        for (String s : new String[]{"", "octave_-7", "minecraft:overworld", "minecraft:aquifer"}) {
            RandomSupport.Seed128bit hs = RandomSupport.seedFromHashOf(s);
            JsonObject o = new JsonObject();
            o.addProperty("str", s);
            o.addProperty("lo", Long.toString(hs.seedLo()));
            o.addProperty("hi", Long.toString(hs.seedHi()));
            hashSeeds.add(o);
        }
        root.add("seed_from_hash", hashSeeds);

        JsonArray legacy = new JsonArray();
        JsonArray xoro = new JsonArray();
        for (long s : new long[]{0L, 1L, -1L, 123456789L, 0x123456789abcdef0L}) {
            JsonObject lEntry = new JsonObject();
            lEntry.addProperty("seed", Long.toString(s));
            lEntry.add("data", captureStream(new LegacyRandomSource(s)));
            legacy.add(lEntry);

            JsonObject xEntry = new JsonObject();
            xEntry.addProperty("seed", Long.toString(s));
            xEntry.add("data", captureStream(new XoroshiroRandomSource(s)));
            xoro.add(xEntry);
        }
        root.add("legacy", legacy);
        root.add("xoroshiro", xoro);

        // Zero seed pair test for Xoroshiro128PlusPlus
        JsonObject zeroPair = new JsonObject();
        zeroPair.addProperty("seed_lo", "0");
        zeroPair.addProperty("seed_hi", "0");
        zeroPair.add("data", captureStream(new XoroshiroRandomSource(0L, 0L)));
        root.add("xoroshiro_zero_pair", zeroPair);

        // Slime chunk seed helper
        JsonArray slime = new JsonArray();
        for (int[] c : new int[][]{{0, 0}, {-1, -1}, {123, -456}, {-99999, 88888}}) {
            JsonObject sc = new JsonObject();
            sc.addProperty("cx", c[0]);
            sc.addProperty("cz", c[1]);
            sc.addProperty("world_seed", "987654321012345");
            sc.addProperty("salt", "987234911");
            RandomSource sr = WorldgenRandom.seedSlimeChunk(c[0], c[1], 987654321012345L, 987234911L);
            sc.addProperty("first_int_10", sr.nextInt(10));
            slime.add(sc);
        }
        root.add("slime_chunk", slime);
        return root;
    }

    private static JsonObject genMth() {
        JsonObject root = new JsonObject();
        JsonArray trig = new JsonArray();
        double[] angles = {
            0.0, -0.0, 1.0, -1.0, Math.PI, -Math.PI, Math.PI / 2.0, Math.PI * 2.0,
            1e-7, -1e-7, 1234.5678, -98765.4321, 1e8, -1e8
        };
        for (double a : angles) {
            JsonObject t = new JsonObject();
            t.addProperty("in_bits", f64Hex(a));
            t.addProperty("sin_bits", f32Hex(Mth.sin(a)));
            t.addProperty("cos_bits", f32Hex(Mth.cos(a)));
            trig.add(t);
        }
        // Dense sweep across full 65536 LUT
        for (int i = 0; i < 65536; i += 257) {
            double a = (i / 65536.0) * Math.PI * 2.0;
            JsonObject t = new JsonObject();
            t.addProperty("in_bits", f64Hex(a));
            t.addProperty("sin_bits", f32Hex(Mth.sin(a)));
            t.addProperty("cos_bits", f32Hex(Mth.cos(a)));
            trig.add(t);
        }
        root.add("trig", trig);

        JsonArray getSeeds = new JsonArray();
        int[][] coords = {{0, 0, 0}, {1, 2, 3}, {-1, -2, -3}, {30000000, -64, -30000000}, {123456, 789, -654321}};
        for (int[] c : coords) {
            JsonObject gs = new JsonObject();
            gs.addProperty("x", c[0]);
            gs.addProperty("y", c[1]);
            gs.addProperty("z", c[2]);
            gs.addProperty("seed", Long.toString(Mth.getSeed(c[0], c[1], c[2])));
            getSeeds.add(gs);
        }
        root.add("get_seed", getSeeds);

        JsonArray murmurs = new JsonArray();
        for (int v : new int[]{0, 1, -1, 42, Integer.MIN_VALUE, Integer.MAX_VALUE, 0x12345678}) {
            JsonObject m = new JsonObject();
            m.addProperty("in", v);
            m.addProperty("out", Mth.murmurHash3Mixer(v));
            murmurs.add(m);
        }
        root.add("murmur3", murmurs);

        JsonArray pow2 = new JsonArray();
        for (int v : new int[]{0, 1, 2, 3, 4, 5, 15, 16, 17, 1023, 1024, 1025, 30000000, 1 << 30}) {
            JsonObject p = new JsonObject();
            p.addProperty("in", v);
            p.addProperty("smallest_pow2", Mth.smallestEncompassingPowerOfTwo(v));
            p.addProperty("ceillog2", Mth.ceillog2(Math.max(1, v)));
            p.addProperty("log2", Mth.log2(Math.max(1, v)));
            pow2.add(p);
        }
        root.add("pow2", pow2);

        JsonArray atan2List = new JsonArray();
        double[] vals = {0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 100.0, -100.0, 1e-9, Double.NaN};
        for (double y : vals) {
            for (double x : vals) {
                JsonObject o = new JsonObject();
                o.addProperty("y_bits", f64Hex(y));
                o.addProperty("x_bits", f64Hex(x));
                o.addProperty("out_bits", f64Hex(Mth.atan2(y, x)));
                atan2List.add(o);
            }
        }
        root.add("atan2", atan2List);

        JsonArray interp = new JsonArray();
        double[] dVals = {-1.5, -0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 2.5};
        for (double t : dVals) {
            JsonObject o = new JsonObject();
            o.addProperty("t_bits", f64Hex(t));
            o.addProperty("smoothstep_bits", f64Hex(Mth.smoothstep(t)));
            o.addProperty("smoothstep_deriv_bits", f64Hex(Mth.smoothstepDerivative(t)));
            o.addProperty("lerp_bits", f64Hex(Mth.lerp(t, -12.34, 56.78)));
            o.addProperty("clamped_lerp_bits", f64Hex(Mth.clampedLerp(t, -12.34, 56.78)));
            o.addProperty("floor", Mth.floor(t));
            o.addProperty("lfloor", Long.toString(Mth.lfloor(t)));
            o.addProperty("ceil", Mth.ceil(t));
            interp.add(o);
        }
        root.add("interp", interp);
        return root;
    }

    private static JsonObject genNoise() {
        JsonObject root = new JsonObject();

        // ImprovedNoise
        ImprovedNoise imp = new ImprovedNoise(new XoroshiroRandomSource(42L));
        JsonObject impObj = new JsonObject();
        impObj.addProperty("xo_bits", f64Hex(imp.xo));
        impObj.addProperty("yo_bits", f64Hex(imp.yo));
        impObj.addProperty("zo_bits", f64Hex(imp.zo));
        JsonArray impSamples = new JsonArray();
        double[][] pts = {
            {0.0, 0.0, 0.0, 0.0, 0.0},
            {12.345, -67.89, 101.25, 0.0, 0.0},
            {-500.125, 33.75, 999.5, 0.25, 0.1},
            {10000.5, -12.25, -4321.125, 0.125, 0.5}
        };
        for (double[] pt : pts) {
            JsonObject s = new JsonObject();
            s.addProperty("x_bits", f64Hex(pt[0]));
            s.addProperty("y_bits", f64Hex(pt[1]));
            s.addProperty("z_bits", f64Hex(pt[2]));
            s.addProperty("y_scale_bits", f64Hex(pt[3]));
            s.addProperty("y_max_bits", f64Hex(pt[4]));
            s.addProperty("val_bits", f64Hex(imp.noise(pt[0], pt[1], pt[2], pt[3], pt[4])));
            double[] deriv = new double[]{1.0, -2.0, 3.0};
            double dv = imp.noiseWithDerivative(pt[0], pt[1], pt[2], deriv);
            s.addProperty("deriv_val_bits", f64Hex(dv));
            s.addProperty("d0_bits", f64Hex(deriv[0]));
            s.addProperty("d1_bits", f64Hex(deriv[1]));
            s.addProperty("d2_bits", f64Hex(deriv[2]));
            impSamples.add(s);
        }
        impObj.add("samples", impSamples);
        root.add("improved", impObj);

        // PerlinNoise modern & legacy
        PerlinNoise perlin = PerlinNoise.create(new XoroshiroRandomSource(12345L), -7, new DoubleArrayList(List.of(1.0, 1.0, 2.0, 0.0, 0.5)));
        JsonArray perlinSamples = new JsonArray();
        for (int i = -5; i <= 5; i++) {
            double x = i * 17.375;
            double y = i * -4.125;
            double z = i * 31.625;
            JsonObject s = new JsonObject();
            s.addProperty("x_bits", f64Hex(x));
            s.addProperty("y_bits", f64Hex(y));
            s.addProperty("z_bits", f64Hex(z));
            s.addProperty("val_bits", f64Hex(perlin.getValue(x, y, z)));
            perlinSamples.add(s);
        }
        // Also test wrap() directly at large coords
        JsonObject wrapSample = new JsonObject();
        double wx = 40_000_000.25;
        wrapSample.addProperty("x_bits", f64Hex(wx));
        wrapSample.addProperty("y_bits", f64Hex(64.5));
        wrapSample.addProperty("z_bits", f64Hex(-40_000_000.75));
        wrapSample.addProperty("val_bits", f64Hex(perlin.getValue(wx, 64.5, -40_000_000.75)));
        perlinSamples.add(wrapSample);
        root.add("perlin", perlinSamples);

        // NormalNoise
        NormalNoise normal = NormalNoise.create(new XoroshiroRandomSource(999L), -5, 1.0, 0.0, 1.0, 2.0);
        JsonArray normalSamples = new JsonArray();
        for (int i = -5; i <= 5; i++) {
            double x = i * 11.25;
            double y = i * 3.5;
            double z = i * -19.75;
            JsonObject s = new JsonObject();
            s.addProperty("x_bits", f64Hex(x));
            s.addProperty("y_bits", f64Hex(y));
            s.addProperty("z_bits", f64Hex(z));
            s.addProperty("val_bits", f64Hex(normal.getValue(x, y, z)));
            normalSamples.add(s);
        }
        root.add("normal", normalSamples);

        // SimplexNoise
        SimplexNoise simplex = new SimplexNoise(new LegacyRandomSource(777L));
        JsonArray simplexSamples = new JsonArray();
        for (int i = -5; i <= 5; i++) {
            double x = i * 0.75;
            double y = i * -1.25;
            double z = i * 0.5;
            JsonObject s = new JsonObject();
            s.addProperty("x_bits", f64Hex(x));
            s.addProperty("y_bits", f64Hex(y));
            s.addProperty("z_bits", f64Hex(z));
            s.addProperty("val2d_bits", f64Hex(simplex.getValue(x, y)));
            s.addProperty("val3d_bits", f64Hex(simplex.getValue(x, y, z)));
            simplexSamples.add(s);
        }
        root.add("simplex", simplexSamples);
        return root;
    }

    private static JsonObject genPacking() {
        JsonObject root = new JsonObject();

        JsonArray blockPosList = new JsonArray();
        int[][] bps = {
            {0, 0, 0}, {1, -64, 1}, {-1, -1, -1},
            {30000000, 319, -30000000}, {-30000000, -2048, 30000000},
            {1234567, 2047, -7654321}
        };
        for (int[] b : bps) {
            long packed = BlockPos.asLong(b[0], b[1], b[2]);
            BlockPos back = BlockPos.of(packed);
            JsonObject o = new JsonObject();
            o.addProperty("x", b[0]);
            o.addProperty("y", b[1]);
            o.addProperty("z", b[2]);
            o.addProperty("packed", Long.toString(packed));
            o.addProperty("back_x", back.getX());
            o.addProperty("back_y", back.getY());
            o.addProperty("back_z", back.getZ());
            blockPosList.add(o);
        }
        root.add("block_pos", blockPosList);

        JsonArray secPosList = new JsonArray();
        int[][] sps = {{0, 0, 0}, {-1, -4, -1}, {1875000, 127, -1875000}, {-1875000, -128, 1875000}};
        for (int[] sp : sps) {
            long packed = SectionPos.asLong(sp[0], sp[1], sp[2]);
            SectionPos back = SectionPos.of(packed);
            JsonObject o = new JsonObject();
            o.addProperty("x", sp[0]);
            o.addProperty("y", sp[1]);
            o.addProperty("z", sp[2]);
            o.addProperty("packed", Long.toString(packed));
            o.addProperty("back_x", back.x());
            o.addProperty("back_y", back.y());
            o.addProperty("back_z", back.z());
            secPosList.add(o);
        }
        root.add("section_pos", secPosList);

        JsonArray bitStorage = new JsonArray();
        for (int bits : new int[]{1, 4, 5, 6, 7, 9, 12, 15, 32}) {
            int size = bits <= 8 ? 4096 : 64;
            int[] values = new int[size];
            int mask = bits == 32 ? -1 : (1 << bits) - 1;
            for (int i = 0; i < size; i++) {
                values[i] = (i * 2654435761L > 0 ? (i * 13 + 7) : (i * 17)) & mask;
                if (bits == 32) values[i] = i * 1664525 + 1013904223;
            }
            SimpleBitStorage st = new SimpleBitStorage(bits, size, values);
            JsonObject o = new JsonObject();
            o.addProperty("bits", bits);
            o.addProperty("size", size);
            JsonArray inArr = new JsonArray();
            for (int v : values) inArr.add(v);
            o.add("values", inArr);
            JsonArray rawArr = new JsonArray();
            for (long r : st.getRaw()) rawArr.add(Long.toString(r));
            o.add("raw_longs", rawArr);
            bitStorage.add(o);
        }
        root.add("bit_storage", bitStorage);
        return root;
    }

    private static JsonObject genNbt() throws Exception {
        CompoundTag rootTag = new CompoundTag();
        rootTag.putByte("b", (byte) -42);
        rootTag.putShort("s", (short) -12345);
        rootTag.putInt("i", 0x76543210);
        rootTag.putLong("l", -1234567890123456789L);
        rootTag.putFloat("f", -12.5f);
        rootTag.putFloat("f_neg_zero", -0.0f);
        rootTag.putDouble("d", Math.PI);
        rootTag.put("ba", new ByteArrayTag(new byte[]{0, 1, -1, 127, -128}));
        rootTag.putString("str_ascii", "minecraft:stone");
        rootTag.putString("str_utf8", "café \u0000 \uD83D\uDE80");
        rootTag.put("empty_list", new ListTag());

        ListTag compoundList = new ListTag();
        CompoundTag c1 = new CompoundTag();
        c1.putString("Name", "minecraft:oak_stairs");
        c1.putInt("Count", 64);
        compoundList.add(c1);
        CompoundTag c2 = new CompoundTag();
        c2.putString("Name", "minecraft:redstone");
        c2.putInt("Count", 16);
        compoundList.add(c2);
        rootTag.put("list_compounds", compoundList);

        rootTag.put("ia", new IntArrayTag(new int[]{0, -1, Integer.MAX_VALUE, Integer.MIN_VALUE}));
        rootTag.put("la", new LongArrayTag(new long[]{0L, -1L, Long.MAX_VALUE, Long.MIN_VALUE}));

        // Also test StringTag.valueOf / FloatTag / DoubleTag directly
        rootTag.put("nested", c1.copy());

        ByteArrayOutputStream baos = new ByteArrayOutputStream();
        try (DataOutputStream dos = new DataOutputStream(baos)) {
            NbtIo.write(rootTag, dos);
        }
        byte[] bytes = baos.toByteArray();
        StringBuilder hex = new StringBuilder(bytes.length * 2);
        for (byte b : bytes) {
            hex.append(String.format("%02x", b));
        }

        JsonObject out = new JsonObject();
        out.addProperty("uncompressed_hex", hex.toString());
        out.addProperty("str_with_nul_and_emoji", "café \u0000 \uD83D\uDE80");
        return out;
    }
}
