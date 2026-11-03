// Full126e0 with real17400/124f0/13340/41cbd0/41c790/12640, no stubs.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeFactoryPlacementValidationOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,B=0x5c3f0cL,F=0x5c5e38L,O=0x612f80L,P=0x58aaaaL,T=0x71000000L,Q=0x72000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(B),new byte[64*30]);e.writeMemory(toAddr(F),new byte[35*88]);e.writeMemory(toAddr(O),new byte[200*44]);e.writeMemory(toAddr(P),new byte[16384]);for(int i=0;i<64;i++){put(e,B+i*30,2,-1);put(e,0x5a1d5e+i*6,2,-1);}for(int i=0;i<35;i++)put(e,F+i*88+4,1,255);for(int i=0;i<200;i++)put(e,O+i*44,2,-1);
   int x=(s*13%80+10)*16,y=(s*7%80+10)*16,kind=new int[]{-1,0,1,2,8}[(s>>3)%5],bd=new int[]{-16,0,32}[(s>>5)%3],gd=new int[]{-48,-32,-16,0,16,32,48,64}[(s>>2)&7],od=new int[]{-16,0,16,32,48}[(s>>4)%5],extent=new int[]{-1,0,8,16,32,64,127}[(s>>3)%7];
   put(e,B,2,0);put(e,B+2,2,0);put(e,B+4,2,128);put(e,B+6,2,128);put(e,0x61b650,4,(s&1)!=0?B+30:B);put(e,0x4a69ea,1,1);put(e,0x4a767c,2,(s&2)!=0?x:-500);put(e,0x4a766c,2,(s&2)!=0?y:-500);put(e,0x622cb8,2,s%5*16);put(e,0x622cba,2,s%3*16);
   put(e,F,2,x+bd);put(e,F+2,2,y+bd);put(e,F+4,1,kind);put(e,F+30,2,0);put(e,U+0x31,1,(s&32)!=0?5:1);put(e,U+0x7c,1,2);put(e,U+0x2c,2,x+gd);put(e,U+0x2e,2,y+gd);
   put(e,O,2,x+od);put(e,O+2,2,y+od);put(e,O+4,4,T);put(e,T+20,1,extent);put(e,T+21,1,(s&64)!=0?8:extent);if((s&128)!=0){put(e,0x5a1d5e,2,x-4);put(e,0x5a1d60,2,y-8);}if((s&4)!=0)put(e,P+(y>>4)*128+(x>>4),1,1);
   put(e,Q,4,x+7);put(e,Q+4,4,y+9);e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,45);put(e,S+8,4,s%19);e.writeRegister("ECX",Q);e.writeRegister("EDX",Q+4);if(!e.run(toAddr(0x4126e0),null,monitor))throw new Exception(e.getLastError());
   out.println("{\"scenario\":"+s+",\"failed\":"+(e.readRegister("EAX").intValue()&255)+",\"x\":"+(short)read(e,Q,2)+",\"y\":"+(short)read(e,Q+4,2)+"}");
  }finally{e.dispose();}
 }}}
}
