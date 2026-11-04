// Execute each original path-worker callback; only terrain seed/walk/final
// callbacks are explicit recorded boundaries. Queue timing and clears execute.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRouteWorkerOracle extends GhidraScript {
 static final long U=0x5e30c8L,J=0x4d131cL,M=0x72000000L,T=0x70000000L,SP=0x7fff0000L,RET=0x12345000L;
 static final long[] FS={0x43c4c0,0x43c5a0,0x43c770,0x43c680}, VS={0x43da20,0x43d7f0,0x43d8a0,0x43d960}, VW={0x43d7c0,0x43d790,0x43d760,0x43d730};
 void put(EmulatorHelper e,long a,int n,long v)throws Exception {e.writeMemoryValue(toAddr(a),n,v);}
 long read(EmulatorHelper e,long a,int n) {byte[] b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 int hash(byte[] bytes){int h=0x811c9dc5;for(byte b:bytes)h=(h^(b&255))*16777619;return h;}
 int add(int h,int v){return(h^v)*16777619;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<64;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{
   boolean vehicle=(s&1)!=0;int trace=0x811c9dc5,direction=0,finishes=0;
   e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(J),new byte[64]);e.writeMemory(toAddr(T),new byte[128]);
   put(e,U+0x7c,1,vehicle?1:0);put(e,U+0x2c,2,500+s%16);put(e,U+0x2e,2,600+s%8);put(e,U+0x10,4,4000+s);put(e,U+0xb4,1,(s>>2)&1);put(e,U+0x67,1,17+s);put(e,U+0x18,4,T);
   put(e,0x63288c,4,U);put(e,0x6338dc,4,0x43e4a0);put(e,0x4d12f8,2,0);put(e,0x586862,1,127);put(e,0x58685e,1,0);put(e,0x633948,4,M);
   byte[] mask=new byte[16384],headers=new byte[400];for(int i=0;i<mask.length;i++)mask[i]=(byte)(s*13+i*7);for(int i=0;i<headers.length;i++)headers[i]=(byte)(s*19+i*3);e.writeMemory(toAddr(M),mask);e.writeMemory(toAddr(0x4d15f0),headers);
   e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x43cb20));e.setBreakpoint(toAddr(0x43e190));e.setBreakpoint(toAddr(0x43ca50));e.setBreakpoint(toAddr(0x43c8d0));for(long a:FS)e.setBreakpoint(toAddr(a));for(long a:VS)e.setBreakpoint(toAddr(a));for(long a:VW)e.setBreakpoint(toAddr(a));
   for(int tick=0;tick<160;tick++){
    long function=read(e,0x6338dc,4);if(function==0x40c810)break;e.writeRegister("ESP",SP);put(e,SP,4,RET);put(e,SP+4,4,J);
    if(!e.run(toAddr(function),null,monitor))throw new Exception(e.getLastError());int guard=0;
    while(e.readRegister("EIP").longValue()!=RET){if(++guard>25)throw new Exception("boundary loop");long pc=e.readRegister("EIP").longValue();int result=0;boolean recognized=false;
     for(int d=0;d<4;d++){if(pc==FS[d]||pc==VS[d]){direction=d;trace=add(trace,10+d+(pc==VS[d]?100:0));result=(s+d*3)%5==0?1:0;recognized=true;}if(pc==VW[d]){direction=d;trace=add(trace,120+d);result=(s+tick+d)%11!=0?1:0;recognized=true;}}
     if(pc==0x43c8d0){trace=add(trace,20+direction);result=(s+tick+direction)%11!=0?1:0;recognized=true;}
     if(pc==0x43cb20){trace=add(trace,2);result=s%17==0?1:0;recognized=true;}
     if(pc==0x43e190){trace=add(trace,30);if(vehicle&&s%7==0&&finishes++==0){put(e,0x4d12f8,2,0);put(e,0x586862,1,2);}recognized=true;}
     if(pc==0x43ca50){trace=add(trace,31);recognized=true;}if(!recognized)throw new Exception("unexpected "+Long.toHexString(pc));
     e.writeRegister("EAX",result);long stack=e.readRegister("ESP").longValue(),next=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(next),null,monitor))throw new Exception(e.getLastError());
    }
    out.println("{\"scenario\":"+s+",\"tick\":"+tick+",\"function\":"+read(e,0x6338dc,4)+",\"phase\":"+(short)read(e,0x4d12f8,2)+",\"grace\":"+read(e,0x586862,1)+",\"countdown\":"+(byte)read(e,0x58685e,1)+",\"cursor\":"+read(e,J+24,4)+",\"origin\":"+(short)read(e,0x4d15e4,2)+",\"b4\":"+read(e,U+0xb4,1)+",\"counter67\":"+read(e,U+0x67,1)+",\"maskHash\":"+Integer.toUnsignedString(hash(e.readMemory(toAddr(M),16384)))+",\"headerHash\":"+Integer.toUnsignedString(hash(e.readMemory(toAddr(0x4d15f0),400)))+",\"trace\":"+Integer.toUnsignedString(trace)+"}");
   }
  }finally{e.dispose();}
 }}}
}
