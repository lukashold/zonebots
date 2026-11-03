// Executes the original APC manual burst loop and its unit-target gate.
// Scan/direction and final projectiles are intercepted; RNG and phase gates run.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeApcManualOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,G=0x60ec00L,W=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int kind:new int[]{3,15})for(int multi=0;multi<2;multi++)for(int passengers:new int[]{1,6,21})for(int phase=0;phase<16;phase++)for(int facing:new int[]{0,3}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{
    e.writeMemory(toAddr(U),new byte[0xc0*32]);e.writeMemory(toAddr(G),new byte[0x48]);e.writeMemory(toAddr(W),new byte[0x30]);
    put(e,U+8,4,W);put(e,U+0x2c,2,500);put(e,U+0x2e,2,600);put(e,U+0x39,1,facing);put(e,U+0x8e,1,kind);put(e,U+0x78,1,70);put(e,W+0x20,1,70);put(e,G+0x12,1,passengers);put(e,0x622dc4,1,multi);
    for(int i=0;i<5;i++)put(e,0x622cdc+i*4,4,i+1);
    for(int p=0;p<21;p++){put(e,G+0x13+p,1,p<passengers?p+1:255);put(e,U+(p+1)*0xc0+0xb6,1,phase);}
    for(long a:new long[]{RET,0x459af0,0x4648a0,0x464a20})e.setBreakpoint(toAddr(a));
    e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,900);e.writeRegister("ECX",U);e.writeRegister("EDX",800);
    if(!e.run(toAddr(0x447da0),null,monitor))throw new Exception(e.getLastError());int shot=0;
    while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue(),s=e.readRegister("ESP").longValue();
     if(a==0x459af0)e.writeRegister("EAX",2);
     else if(a==0x4648a0||a==0x464a20){out.println("{\"kind\":\"shot\",\"class\":"+kind+",\"multi\":"+multi+",\"passengers\":"+passengers+",\"phase\":"+phase+",\"facing\":"+facing+",\"shot\":"+(shot++)+",\"x\":"+read(e,s+4,4)+",\"y\":"+read(e,s+8,4)+",\"tx\":"+read(e,s+12,4)+",\"ty\":"+read(e,s+16,4)+"}");}
     else throw new Exception("Unexpected manual APC stop");resume(e);
    }
    out.println("{\"kind\":\"burst\",\"class\":"+kind+",\"multi\":"+multi+",\"passengers\":"+passengers+",\"phase\":"+phase+",\"facing\":"+facing+",\"shots\":"+shot+",\"reload\":"+read(e,U+0x7b,1)+",\"nextPhase\":"+read(e,U+0xc0+0xb6,1)+"}");
   }finally{e.dispose();}
  }
  for(int kind:new int[]{0,3,15,16})for(int marker:new int[]{-1234,-4321,100}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);
   try{e.writeMemory(toAddr(U),new byte[0xc0]);e.writeMemory(toAddr(W),new byte[0x30]);put(e,U+0x8e,1,kind);put(e,W+0xe,2,marker);e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x45b4f0));
    e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);e.writeRegister("EDX",W);if(!e.run(toAddr(0x45ac40),null,monitor))throw new Exception(e.getLastError());
    out.println("{\"kind\":\"gate\",\"class\":"+kind+",\"marker\":"+marker+",\"accepted\":"+(e.readRegister("EIP").longValue()==0x45b4f0)+"}");
   }finally{e.dispose();}
  }
 }}
}
