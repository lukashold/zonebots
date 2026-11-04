//40cb20 caller with actual40c9b0 dish queue/health/clock. The other radar
//parts, fragment updater, final raster calls and audio wrappers are boundaries.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.PrintWriter;
public class ZNativeRadarDisplayOracle extends GhidraScript{
 static final long B=0x5c5e38L,CP=0x5c3f0cL,Q=0x62dc50L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(EmulatorHelper e,int n){int h=0x811c9dc5;if(n>0)for(byte b:e.readMemory(toAddr(Q),n))h=(h^(b&255))*16777619;return h;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{RET,0x4666c0,0x41f610,0x40c8a0,0x40c930,0x40ca70,0x46e6d0,0x46f650})e.setBreakpoint(toAddr(a));put(e,0x622cb8,4,0);put(e,0x498c24,2,1024);put(e,0x498c20,2,1024);put(e,0x622dc2,1,0);put(e,0x62dc34,2,0);e.writeMemory(toAddr(0x622df9),new byte[]{0,1,2,3,4});
  for(int s=0;s<4096;s++){int frame=s%8,owner=new int[]{0,1,3,4,-1,127}[s/8%6],max=new int[]{-32768,-1,0,1,9,100,32767}[s/48%7],threshold=max-(max/5)*4,hp=new int[]{0,threshold-1,threshold,threshold+1,32767}[s/336%5],pulse=s/1680%2,x=new int[]{-64,-63,500,1024,1025}[s*7%5],y=new int[]{-48,-47,500,1024,1025}[(s*3+1)%5],flags=(s*13)&255,notice=(s*7)&255;
   e.writeMemory(toAddr(B),new byte[88]);e.writeMemory(toAddr(Q),new byte[16]);put(e,B,2,x);put(e,B+2,2,y);put(e,B+4,1,8);put(e,B+6,1,flags);put(e,B+10,2,frame);put(e,B+21,1,notice);put(e,B+28,2,max);put(e,B+30,2,hp);put(e,B+32,4,CP);put(e,CP+8,1,owner);put(e,0x622dc0,1,pulse);put(e,0x62b3a4,4,Q);put(e,S,4,RET);put(e,S+4,4,B);e.writeRegister("ESP",S);int sweep=0,secured=0;if(!e.run(toAddr(0x40cb20),null,monitor))throw new Exception(e.getLastError());
   for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){if(budget>14)throw new Exception("unexpected calls");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(pc==0x46e6d0){if(e.readRegister("ECX").intValue()!=16)throw new Exception("wrong sweep sample");sweep++;}else if(pc==0x46f650)secured++;else if(pc!=0x4666c0&&pc!=0x41f610&&pc!=0x40c8a0&&pc!=0x40c930&&pc!=0x40ca70)throw new Exception("unexpected "+Long.toHexString(pc));long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
   int size=(int)(read(e,0x62b3a4,4)-Q);out.println(s+","+frame+","+owner+","+max+","+hp+","+pulse+","+x+","+y+","+flags+","+notice+","+(short)read(e,B+10,2)+","+read(e,B+6,1)+","+read(e,B+21,1)+","+size+","+(hash(e,size)&0xffffffffL)+","+sweep+","+secured);
  }
 }finally{e.dispose();}}}
}
