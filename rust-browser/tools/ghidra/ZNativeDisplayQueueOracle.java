// Full545b0/4549f0/40bb50 queue and palette sequence. Only final texture,
//palette upload and draw submissions are intercepted; no sort is substituted.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeDisplayQueueOracle extends GhidraScript{
 static final long Q=0x62dc50L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int mix(int h,int...values){for(int v:values)h=(h^v)*16777619;return h;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{RET,0x46bde0,0x465cb0,0x466090,0x466180})e.setBreakpoint(toAddr(a));
  for(int scenario=0;scenario<512;scenario++){
   int count=scenario==0?0:1+scenario%17,mode=scenario%3,forced=scenario/3%2,current=new int[]{0,4,255}[scenario/6%3],uploaded=new int[]{0,1,4,255}[scenario/18%4];e.writeMemory(toAddr(Q),new byte[17*8]);
   for(int i=0;i<count;i++){int y=scenario%4==0?i%4:scenario%4==1?count-i:scenario%4==2?(short)(i*7313+scenario*37):-100;int palette=new int[]{0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,255}[(scenario+i*7)%17];int dy=new int[]{-128,-1,0,16,127}[(scenario+i)%5];if(palette==255&&dy==-1)dy=0;put(e,Q+i*8,2,100+i);put(e,Q+i*8+2,2,y);put(e,Q+i*8+4,2,1000+i);put(e,Q+i*8+6,1,palette);put(e,Q+i*8+7,1,dy);}
   put(e,0x62b3a4,4,Q+count*8);put(e,0x680cfc,4,mode);put(e,0x4d02df,1,forced);put(e,0x477063,1,current);put(e,0x47717f,1,uploaded);put(e,S,4,RET);e.writeRegister("ESP",S);int trace=0x811c9dc5,uploads=0,draws=0;
   if(!e.run(toAddr(0x4549f0),null,monitor))throw new Exception(e.getLastError());
   for(int guard=0;e.readRegister("EIP").longValue()!=RET;guard++){if(guard>count*4+1)throw new Exception("call budget");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();int pal=(int)read(e,0x477063,1);if(pc==0x46bde0){uploads++;trace=mix(trace,0,pal);}else if(pc==0x465cb0){e.writeRegister("ECX",0);}else if(pc==0x466090||pc==0x466180){draws++;trace=mix(trace,pc==0x466090?1:2,(short)e.readRegister("ECX").intValue(),(short)e.readRegister("EDX").intValue(),(short)read(e,sp+8,2),pal);}else throw new Exception("unexpected "+Long.toHexString(pc));long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
   int bytes=0x811c9dc5;if(count>0)for(byte b:e.readMemory(toAddr(Q),count*8))bytes=mix(bytes,b&255);out.println(scenario+","+count+","+(bytes&0xffffffffL)+","+read(e,0x477063,1)+","+read(e,0x47717f,1)+","+(trace&0xffffffffL)+","+uploads+","+draws);
  }
 }finally{e.dispose();}}}
}
