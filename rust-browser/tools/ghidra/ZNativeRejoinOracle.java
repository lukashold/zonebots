// Execute real4e910/54070/540b0/52500/4d430; intercept only final path emission.
// Generated signed trigonometry/mask fixtures isolate exact integer geometry.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRejoinOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,TRIG=0x71000000L,MASK=0x72000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int scenario=0;scenario<256;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);int count=0,x=0,y=0;
   try{
    e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[240*72]);e.writeMemory(toAddr(0x612f80),new byte[200*44]);
    for(int i=0;i<240;i++)for(int m=0;m<21;m++)put(e,R+i*72+19+m,1,255);
    put(e,R+19,1,2);put(e,R+20,1,3);put(e,U+2*192+0x8a,1,(scenario&1)!=0?32:0);put(e,U+3*192+0x8a,1,(scenario&2)!=0?32:0);
    long g=R+72,u=U+192;put(e,g+19,1,0);put(e,g+20,1,1);put(e,g+18,1,2);put(e,g+9,1,0x46);put(e,g+10,2,scenario%5==0?513:(scenario*71)%513);put(e,g+50,2,100);put(e,g+56,2,200);
    for(int i=0;i<640;i++)put(e,TRIG+i*2,2,(i*73)%2048-1024);put(e,0x586a80,4,TRIG);put(e,0x49a050,4,MASK);
    for(int i=0;i<20;i++){put(e,MASK+i*2,1,(i*13)%255-127);put(e,MASK+i*2+1,1,(i*29)%255-127);}
    int[]roles={-1,0,1,20,3,2,4,1};put(e,u+0x5e,1,roles[scenario%8]);put(e,u+0x5f,1,1);put(e,u+0x31,1,5|32|((scenario&32)!=0?8:0));put(e,u+0x9c,1,(scenario&16)!=0?4:0);put(e,u+0x8a,1,(scenario&8)!=0?16:0);put(e,u+0x2c,2,(scenario&128)!=0?100:0);put(e,u+0x2e,2,(scenario&128)!=0?200:0);put(e,U+0x2c,2,(scenario&64)!=0?32760:500);put(e,U+0x2e,2,(scenario&64)!=0?-32760:800);
    put(e,u+0x14,4,0x5eee28);put(e,0x5eee28+4,4,0);put(e,u+12,4,0x5f8428);put(e,0x5f8428+4,4,0);put(e,0x632194,4,R);put(e,0x4d029b,1,(scenario&4)!=0?1:0);put(e,0x622d1a,2,0);put(e,0x612f80,2,65535);
    for(long a:new long[]{RET,0x459c60})e.setBreakpoint(toAddr(a));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",u);if(!e.run(toAddr(0x44e910),null,monitor))throw new Exception(e.getLastError());
    while(e.readRegister("EIP").longValue()!=RET){if(e.readRegister("EIP").longValue()!=0x459c60)throw new Exception("Unexpected rejoin stop");count++;x=(short)e.readRegister("EDX").intValue();y=(short)read(e,e.readRegister("ESP").longValue()+4,4);e.writeRegister("EAX",0);resume(e);}
    out.println("{\"scenario\":"+scenario+",\"routes\":"+count+",\"x\":"+x+",\"y\":"+y+",\"primary\":"+(read(e,u+0x14,4)!=0?1:0)+",\"secondary\":"+(read(e,u+12,4)!=0?1:0)+",\"flags31\":"+read(e,u+0x31,1)+",\"angle\":"+(short)read(e,g+10,2)+",\"special\":"+(read(e,u+0x35,1)!=0?1:0)+",\"flags9\":"+read(e,g+9,1)+"}");
   }finally{e.dispose();}
  }
 }}
}
