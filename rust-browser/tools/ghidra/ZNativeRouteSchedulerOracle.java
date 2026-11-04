//43e780 executes fully. The selected43e4a0/43dad0 path worker is the explicit
//recorded boundary; its entry masks are captured before returning unchanged.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRouteSchedulerOracle extends GhidraScript{
 static final long RET=0x12345000L,SP=0x7fff0000L,U=0x5e30c8L,T=0x70000000L,C=0x71000000L,R=0x60ec00L;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{
   e.writeMemory(toAddr(U),new byte[240*192]);byte[]table=new byte[240*128];for(int i=0;i<240;i++)for(int j=0;j<128;j++)table[i*128+j]=(byte)(s*17+i*3+j*13);e.writeMemory(toAddr(T),table.clone());for(int i=0;i<table.length;i++)table[i]+=41;e.writeMemory(toAddr(C),table);
   for(int i=0;i<240;i++){long p=U+i*192;int flags=(i+s)&3;if((i+s)%3!=0)flags|=4;if((i*7+s)%17==0)flags|=64;put(e,p+0x31,1,flags);put(e,p+0x5f,1,(i*13+s)%240);put(e,p+0x30,1,new int[]{4,24,30,32,34,36,38}[i%7]);put(e,p+0x7c,1,new int[]{0,1,2,3,15}[i%5]);put(e,p+0x18,4,(i+s)%5==0?0:T+i*128);put(e,p+0x1c,4,C+i*128);}
   int current=s%240;long function=new long[]{0x40c810,0x43e4a0,0x43dad0}[s%3];put(e,0x63288c,4,U+current*192);put(e,0x6338dc,4,function);put(e,0x632194,4,0);put(e,0x4d12f8,2,s*11);put(e,0x586862,1,s*13);
   e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(0x43e4a0));e.setBreakpoint(toAddr(0x43dad0));e.writeRegister("ESP",SP);put(e,SP,4,RET);
   if(!e.run(toAddr(0x43e780),null,monitor))throw new Exception(e.getLastError());int called=e.readRegister("EIP").longValue()==RET?0:1;int[]before=new int[7];java.util.Arrays.fill(before,-1);
   if(called!=0){for(int i=0;i<5;i++)before[i]=(int)read(e,T+current*128+24+i,1);for(int i=0;i<2;i++)before[i+5]=(int)read(e,C+current*128+24+i,1);long stack=e.readRegister("ESP").longValue(),next=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(next),null,monitor))throw new Exception(e.getLastError());}
   int now=(int)((read(e,0x63288c,4)-U)/192);long group=read(e,0x632194,4);int[]after=new int[7];for(int i=0;i<5;i++)after[i]=(int)read(e,T+current*128+24+i,1);for(int i=0;i<2;i++)after[i+5]=(int)read(e,C+current*128+24+i,1);
   out.print("{\"scenario\":"+s+",\"current\":"+now+",\"function\":"+read(e,0x6338dc,4)+",\"phase\":"+(short)read(e,0x4d12f8,2)+",\"grace\":"+read(e,0x586862,1)+",\"group\":"+(group==0?-1:(group-R)/72)+",\"called\":"+called+",\"before\":[");for(int i=0;i<7;i++)out.print((i==0?"":",")+before[i]);out.print("],\"after\":[");for(int i=0;i<7;i++)out.print((i==0?"":",")+after[i]);out.println("]}");
  }finally{e.dispose();}
 }}}
}
