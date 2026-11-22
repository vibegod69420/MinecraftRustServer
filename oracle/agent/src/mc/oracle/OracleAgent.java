package mc.oracle;

import java.lang.instrument.ClassFileTransformer;
import java.lang.instrument.Instrumentation;
import java.security.ProtectionDomain;
import org.objectweb.asm.ClassReader;
import org.objectweb.asm.ClassVisitor;
import org.objectweb.asm.ClassWriter;
import org.objectweb.asm.MethodVisitor;
import org.objectweb.asm.Opcodes;

/**
 * Java agent making the 1.21.11 vanilla server deterministic and tracing RandomSource calls.
 */
public final class OracleAgent {
    private static final int API = Opcodes.ASM9;
    private static final String RUNTIME = "mc/oracle/OracleRuntime";

    private OracleAgent() {}

    public static void premain(String agentArgs, Instrumentation inst) {
        if (!OracleRuntime.DETERMINISTIC && !OracleRuntime.TRACE_ENABLED && !OracleRuntime.START_FROZEN) {
            return;
        }
        inst.addTransformer(new OracleTransformer(), false);
    }

    private static final class OracleTransformer implements ClassFileTransformer {
        @Override
        public byte[] transform(
            ClassLoader loader,
            String className,
            Class<?> classBeingRedefined,
            ProtectionDomain protectionDomain,
            byte[] classfileBuffer
        ) {
            if (className == null) {
                return null;
            }
            boolean isMc = className.startsWith("net/minecraft/") || className.startsWith("com/mojang/");
            if (!isMc) {
                return null;
            }

            boolean isRandomSupport = className.equals("net/minecraft/world/level/levelgen/RandomSupport");
            boolean isRandomSource = className.equals("net/minecraft/util/RandomSource");
            boolean isMinecraftServer = className.equals("net/minecraft/server/MinecraftServer");
            boolean isTracedRng = OracleRuntime.TRACE_ENABLED && (
                className.equals("net/minecraft/world/level/levelgen/LegacyRandomSource")
                || className.equals("net/minecraft/world/level/levelgen/XoroshiroRandomSource")
                || className.equals("net/minecraft/world/level/levelgen/SingleThreadedRandomSource")
                || className.equals("net/minecraft/world/level/levelgen/ThreadSafeLegacyRandomSource")
            );

            ClassReader cr = new ClassReader(classfileBuffer);
            int flags = isTracedRng ? ClassWriter.COMPUTE_FRAMES : ClassWriter.COMPUTE_MAXS;
            ClassWriter cw = new ClassWriter(cr, flags) {
                @Override
                protected String getCommonSuperClass(String type1, String type2) {
                    try {
                        return super.getCommonSuperClass(type1, type2);
                    } catch (Throwable ignored) {
                        return "java/lang/Object";
                    }
                }
            };
            ClassVisitor cv = new OracleClassVisitor(
                cw, className, isRandomSupport, isRandomSource, isMinecraftServer, isTracedRng
            );
            cr.accept(cv, isTracedRng ? ClassReader.EXPAND_FRAMES : 0);
            return cw.toByteArray();
        }
    }

    private static final class OracleClassVisitor extends ClassVisitor {
        private final String className;
        private final boolean isRandomSupport;
        private final boolean isRandomSource;
        private final boolean isMinecraftServer;
        private final boolean isTracedRng;
        private boolean addedStreamField = false;

        OracleClassVisitor(
            ClassVisitor cv,
            String className,
            boolean isRandomSupport,
            boolean isRandomSource,
            boolean isMinecraftServer,
            boolean isTracedRng
        ) {
            super(API, cv);
            this.className = className;
            this.isRandomSupport = isRandomSupport;
            this.isRandomSource = isRandomSource;
            this.isMinecraftServer = isMinecraftServer;
            this.isTracedRng = isTracedRng;
        }

        @Override
        public void visit(int version, int access, String name, String signature, String superName, String[] interfaces) {
            super.visit(version, access, name, signature, superName, interfaces);
            if (isTracedRng && !addedStreamField) {
                addedStreamField = true;
                super.visitField(Opcodes.ACC_PRIVATE | Opcodes.ACC_FINAL, "__oracleStreamId", "J", null, null);
            }
        }

        @Override
        public MethodVisitor visitMethod(int access, String name, String descriptor, String signature, String[] exceptions) {
            MethodVisitor mv = super.visitMethod(access, name, descriptor, signature, exceptions);

            if (OracleRuntime.DETERMINISTIC && isRandomSupport && name.equals("generateUniqueSeed") && descriptor.equals("()J")) {
                MethodVisitor gen = mv;
                gen.visitCode();
                gen.visitMethodInsn(Opcodes.INVOKESTATIC, RUNTIME, "nextUniqueSeed", "()J", false);
                gen.visitInsn(Opcodes.LRETURN);
                gen.visitMaxs(2, 0);
                gen.visitEnd();
                return null;
            }

            if (OracleRuntime.DETERMINISTIC && isRandomSource && name.equals("createNewThreadLocalInstance")
                && descriptor.equals("()Lnet/minecraft/util/RandomSource;")) {
                mv.visitCode();
                mv.visitTypeInsn(Opcodes.NEW, "net/minecraft/world/level/levelgen/SingleThreadedRandomSource");
                mv.visitInsn(Opcodes.DUP);
                mv.visitMethodInsn(Opcodes.INVOKESTATIC, RUNTIME, "nextUniqueSeed", "()J", false);
                mv.visitMethodInsn(
                    Opcodes.INVOKESPECIAL,
                    "net/minecraft/world/level/levelgen/SingleThreadedRandomSource",
                    "<init>",
                    "(J)V",
                    false
                );
                mv.visitInsn(Opcodes.ARETURN);
                mv.visitMaxs(4, 0);
                mv.visitEnd();
                return null;
            }

            if (OracleRuntime.START_FROZEN && isMinecraftServer && name.equals("runServer") && descriptor.equals("()V")) {
                mv = new MethodVisitor(API, mv) {
                    private boolean injected = false;

                    @Override
                    public void visitMethodInsn(int opcode, String owner, String mname, String desc, boolean isInterface) {
                        super.visitMethodInsn(opcode, owner, mname, desc, isInterface);
                        if (!injected && mname.equals("initServer") && desc.equals("()Z")) {
                            injected = true;
                            super.visitVarInsn(Opcodes.ALOAD, 0);
                            super.visitMethodInsn(
                                Opcodes.INVOKESTATIC, RUNTIME, "maybeFreezeServer", "(Ljava/lang/Object;)V", false
                            );
                        }
                    }
                };
            }

            if (isTracedRng) {
                mv = wrapTracedRngMethod(mv, name, descriptor);
            }

            if (OracleRuntime.DETERMINISTIC) {
                mv = new DeterministicCallsVisitor(mv);
            }
            return mv;
        }

        private MethodVisitor wrapTracedRngMethod(MethodVisitor mv, String name, String descriptor) {
            final String shortImpl = className.substring(className.lastIndexOf('/') + 1);

            if (name.equals("<init>")) {
                return new MethodVisitor(API, mv) {
                    private boolean assigned = false;

                    @Override
                    public void visitMethodInsn(int opcode, String owner, String mname, String desc, boolean isInterface) {
                        super.visitMethodInsn(opcode, owner, mname, desc, isInterface);
                        if (!assigned && opcode == Opcodes.INVOKESPECIAL && mname.equals("<init>") && owner.equals("java/lang/Object")) {
                            assigned = true;
                            super.visitVarInsn(Opcodes.ALOAD, 0);
                            super.visitMethodInsn(Opcodes.INVOKESTATIC, RUNTIME, "nextStreamId", "()J", false);
                            super.visitFieldInsn(Opcodes.PUTFIELD, className, "__oracleStreamId", "J");
                        }
                    }
                };
            }

            if (name.equals("setSeed") && descriptor.equals("(J)V")) {
                return new MethodVisitor(API, mv) {
                    @Override
                    public void visitCode() {
                        super.visitCode();
                        super.visitVarInsn(Opcodes.ALOAD, 0);
                        super.visitFieldInsn(Opcodes.GETFIELD, className, "__oracleStreamId", "J");
                        super.visitLdcInsn(shortImpl);
                        super.visitLdcInsn("setSeed");
                        super.visitVarInsn(Opcodes.LLOAD, 1);
                        super.visitInsn(Opcodes.LCONST_0);
                        super.visitMethodInsn(
                            Opcodes.INVOKESTATIC,
                            RUNTIME,
                            "logRng",
                            "(JLjava/lang/String;Ljava/lang/String;JJ)V",
                            false
                        );
                    }
                };
            }

            if (name.equals("next") && descriptor.equals("(I)I")) {
                return new MethodVisitor(API, mv) {
                    @Override
                    public void visitInsn(int opcode) {
                        if (opcode == Opcodes.IRETURN) {
                            super.visitVarInsn(Opcodes.ALOAD, 0);
                            super.visitFieldInsn(Opcodes.GETFIELD, className, "__oracleStreamId", "J");
                            super.visitLdcInsn(shortImpl);
                            super.visitLdcInsn("next");
                            super.visitVarInsn(Opcodes.ILOAD, 1);
                            super.visitMethodInsn(
                                Opcodes.INVOKESTATIC,
                                RUNTIME,
                                "logRngInt",
                                "(IJLjava/lang/String;Ljava/lang/String;I)I",
                                false
                            );
                        }
                        super.visitInsn(opcode);
                    }
                };
            }

            if (className.endsWith("XoroshiroRandomSource") && name.equals("nextLong") && descriptor.equals("()J")) {
                return new MethodVisitor(API, mv) {
                    @Override
                    public void visitInsn(int opcode) {
                        if (opcode == Opcodes.LRETURN) {
                            super.visitVarInsn(Opcodes.ALOAD, 0);
                            super.visitFieldInsn(Opcodes.GETFIELD, className, "__oracleStreamId", "J");
                            super.visitLdcInsn(shortImpl);
                            super.visitLdcInsn("nextLong");
                            super.visitInsn(Opcodes.LCONST_0);
                            super.visitMethodInsn(
                                Opcodes.INVOKESTATIC,
                                RUNTIME,
                                "logRngLong",
                                "(JJLjava/lang/String;Ljava/lang/String;J)J",
                                false
                            );
                        }
                        super.visitInsn(opcode);
                    }
                };
            }

            if (className.endsWith("XoroshiroRandomSource") && name.equals("nextInt") && descriptor.equals("()I")) {
                return new MethodVisitor(API, mv) {
                    @Override
                    public void visitInsn(int opcode) {
                        if (opcode == Opcodes.IRETURN) {
                            super.visitVarInsn(Opcodes.ALOAD, 0);
                            super.visitFieldInsn(Opcodes.GETFIELD, className, "__oracleStreamId", "J");
                            super.visitLdcInsn(shortImpl);
                            super.visitLdcInsn("nextInt");
                            super.visitInsn(Opcodes.ICONST_0);
                            super.visitMethodInsn(
                                Opcodes.INVOKESTATIC,
                                RUNTIME,
                                "logRngInt",
                                "(IJLjava/lang/String;Ljava/lang/String;I)I",
                                false
                            );
                        }
                        super.visitInsn(opcode);
                    }
                };
            }

            if (className.endsWith("XoroshiroRandomSource") && name.equals("nextBits") && descriptor.equals("(I)J")) {
                return new MethodVisitor(API, mv) {
                    @Override
                    public void visitInsn(int opcode) {
                        if (opcode == Opcodes.LRETURN) {
                            super.visitVarInsn(Opcodes.ALOAD, 0);
                            super.visitFieldInsn(Opcodes.GETFIELD, className, "__oracleStreamId", "J");
                            super.visitLdcInsn(shortImpl);
                            super.visitLdcInsn("nextBits");
                            super.visitVarInsn(Opcodes.ILOAD, 1);
                            super.visitInsn(Opcodes.I2L);
                            super.visitMethodInsn(
                                Opcodes.INVOKESTATIC,
                                RUNTIME,
                                "logRngLong",
                                "(JJLjava/lang/String;Ljava/lang/String;J)J",
                                false
                            );
                        }
                        super.visitInsn(opcode);
                    }
                };
            }
            return mv;
        }
    }

    private static final class DeterministicCallsVisitor extends MethodVisitor {
        DeterministicCallsVisitor(MethodVisitor mv) {
            super(API, mv);
        }

        @Override
        public void visitMethodInsn(int opcode, String owner, String name, String descriptor, boolean isInterface) {
            if (opcode == Opcodes.INVOKESTATIC && owner.equals("java/util/UUID")
                && name.equals("randomUUID") && descriptor.equals("()Ljava/util/UUID;")) {
                super.visitMethodInsn(
                    Opcodes.INVOKESTATIC, RUNTIME, "nextDeterministicUuid", "()Ljava/util/UUID;", false
                );
                return;
            }
            if (opcode == Opcodes.INVOKESPECIAL && owner.equals("java/util/Random")
                && name.equals("<init>") && descriptor.equals("()V")) {
                super.visitMethodInsn(Opcodes.INVOKESTATIC, RUNTIME, "nextUniqueSeed", "()J", false);
                super.visitMethodInsn(Opcodes.INVOKESPECIAL, "java/util/Random", "<init>", "(J)V", false);
                return;
            }
            if (opcode == Opcodes.INVOKESTATIC && owner.equals("java/lang/Math")
                && name.equals("random") && descriptor.equals("()D")) {
                super.visitMethodInsn(
                    Opcodes.INVOKESTATIC, RUNTIME, "nextDeterministicMathRandom", "()D", false
                );
                return;
            }
            super.visitMethodInsn(opcode, owner, name, descriptor, isInterface);
        }
    }
}
