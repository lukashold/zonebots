// Observes the native APC initializer preserving passenger weapon attributes.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeApcInitOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,E=U+0x1000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a,long c,long d)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",c);e.writeRegister("EDX",d);if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int[] p:new int[][]{{0,60,15,5,80,15},{3,40,15,40,80,10},{0,120,18,10,80,15},{0,50,15,2,98,15},{15,80,15,20,98,13},{16,40,17,15,80,15}}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(U),new byte[0x2000]);put(e,0x59e6c2,2,65535);put(e,U+0x7c,1,p[0]);put(e,U+2,1,p[1]);put(e,U+0x74,1,p[2]);put(e,U+0x65,1,p[3]);put(e,U+0x78,1,p[4]);put(e,U+0x79,1,p[5]);
    put(e,E+0xe,1,15);put(e,E+0x11,1,5);put(e,E+0x18,1,80);put(e,E+0x19,1,15);
    e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x41b950));invoke(e,0x4581f0,U,E);long w=read(e,U+8,4);put(e,S+4,4,80);invoke(e,0x458090,w,U);while(e.readRegister("EIP").longValue()!=RET){if(e.readRegister("EIP").longValue()!=0x41b950)throw new Exception("Unexpected init stop");resume(e);}
    out.println("{\"crewClass\":"+p[0]+",\"crewAim\":"+p[1]+",\"crewRange\":"+p[2]+",\"crewDamage\":"+p[3]+",\"crewRate\":"+p[4]+",\"crewSpeed\":"+p[5]+",\"class\":"+read(e,U+0x7c,1)+",\"weaponClass\":"+read(e,U+0x8e,1)+",\"aim\":"+read(e,U+2,1)+",\"hardwareRange\":"+read(e,U+0x74,1)+",\"range\":"+read(e,w+5,1)+",\"damage\":"+read(e,U+0x65,1)+",\"rate\":"+read(e,U+0x78,1)+",\"speed\":"+read(e,U+0x79,1)+"}");
   }finally{e.dispose();}
  }
 }}
}
