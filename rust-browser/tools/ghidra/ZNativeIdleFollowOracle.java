// Real4d570 and4d430.52960,525a0 and final path emission are explicit
// boundaries; no formation geometry or state predicates are stubbed.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeIdleFollowOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,T=0x71000000L,M=0x72000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int s=0;s<1024;s++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[384]);e.writeMemory(toAddr(R),new byte[72]);long u=U+192;
    int role=new int[]{-1,0,1,2}[(s>>4)&3],flags=5|128|((s&1)!=0?64:0)|((s&64)!=0?8:0),groupFlags=0x46|((s&8)!=0?1:0),x=(s%3==0?500:(s%3==1?480:400)),y=800;
    put(e,u+0x31,1,flags);put(e,u+0x5e,1,role);put(e,u+0x8a,1,(s&256)!=0?16:0);put(e,u+0x9c,1,(s&512)!=0?16:0);put(e,u+0x8c,1,99);put(e,u+0x35,1,3);put(e,u+0x48,2,s%9);put(e,u+0x2c,2,x);put(e,u+0x2e,2,y);
    put(e,R+9,1,groupFlags);put(e,R+0x3f,1,(s&4)!=0?1:0);put(e,R+19,1,0);put(e,R+50,2,500);put(e,R+56,2,800);put(e,R+10,2,s%5==0?513:s%513);put(e,U+0x2c,2,500);put(e,U+0x2e,2,800);
    for(int i=0;i<640;i++)put(e,T+i*2,2,(i*73)%2048-1024);put(e,0x586a80,4,T);put(e,0x49a050,4,M);put(e,M,1,3);put(e,M+1,1,7);put(e,M+2,1,-9);put(e,M+3,1,11);put(e,0x632194,4,R);put(e,0x4d029b,1,(s&128)!=0?1:0);
    for(long a:new long[]{RET,0x452960,0x459c60,0x452810,0x4525a0,0x450b30})e.setBreakpoint(toAddr(a));
    e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",u);if(!e.run(toAddr(0x44d570),null,monitor))throw new Exception(e.getLastError());
    int route=0,tx=0,ty=0,resume=0,special=0,steps=0;
    while(e.readRegister("EIP").longValue()!=RET){if(++steps>12)throw new Exception("too many stops");long pc=e.readRegister("EIP").longValue();
     if(pc==0x452960){e.writeRegister("EAX",(s&2)!=0?1:0);}
     else if(pc==0x452810){resume++;}
     else if(pc==0x459c60||pc==0x4525a0){route=pc==0x459c60?1:2;tx=(short)e.readRegister("EDX").intValue();ty=(short)read(e,e.readRegister("ESP").longValue()+4,4);e.writeRegister("EAX",s%2);}
     else if(pc==0x450b30){special++;e.writeRegister("EAX",0);}
     else throw new Exception("Unexpected "+Long.toHexString(pc));resume(e);
    }
    out.println("{\"scenario\":"+s+",\"route\":"+route+",\"x\":"+tx+",\"y\":"+ty+",\"resume\":"+resume+",\"special\":"+special+",\"flags31\":"+read(e,u+0x31,1)+",\"bonus\":"+read(e,u+0x8c,1)+",\"phase\":"+read(e,u+0x35,1)+",\"angle\":"+(short)read(e,R+10,2)+"}");
   }finally{e.dispose();}
  }
 }}
}
