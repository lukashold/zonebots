// Ambient100 area/bullet/terrain retirement. The separate404dc0 fragment
//allocator is isolated and logged; all100 raw target records are compared.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAmbientDamageOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,B=0x60b028L;long eventHash;int events;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f,int x,int y)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",x);e.writeRegister("EDX",y);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<100000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;if(pc==0x404dc0){long sp=e.readRegister("ESP").longValue();for(long value:new long[]{e.readRegister("EDX").longValue()&65535,read(e,sp+4,2),read(e,sp+8,2)})eventHash=((eventHash^value)*16777619L)&0xffffffffL;events++;e.writeRegister("ESP",sp+4);e.writeRegister("EIP",read(e,sp,4));continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget");}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int operation=0;operation<3;operation++)for(int scenario=0;scenario<128;scenario++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{byte[]pool=new byte[6200];for(int j=0;j<pool.length;j++)pool[j]=(byte)(j*11+scenario*7);e.writeMemory(toAddr(B),pool);
    for(int i=0;i<100;i++){long r=B+i*62;int x=500+(i*17+scenario*13)%65-32,y=600+(i*29+scenario*7)%65-32;put(e,r,2,(i+scenario)%7==0?-1:x);put(e,r+2,2,y);put(e,r+8,2,(i%3==0?500:x)&65520);put(e,r+10,2,(i%3==0?600:y)&65520);put(e,r+38,2,(i+scenario)%5==0?2:0);put(e,r+53,1,(i+scenario)%11==0?16:0);put(e,r+60,1,(i+scenario)%13==0?6:0);put(e,r+30,1,(i+scenario)%17==0?0:(i+scenario)%19==0?255:4);put(e,r+20,2,(i%2==0?-1:1)*(2000+i*3));}
    events=0;eventHash=2166136261L;put(e,0x584978,4,500);put(e,0x584980,4,600);call(e,operation==0?0x404ec0:operation==1?0x404f80:0x41cb40,500,600);long hash=2166136261L;for(byte value:e.readMemory(toAddr(B),6200))hash=((hash^(value&255L))*16777619L)&0xffffffffL;out.println(operation+","+scenario+","+(operation==1?(e.readRegister("EAX").longValue()&255):0)+","+hash+","+events+","+eventHash);out.flush();
   }finally{e.dispose();}
  }
 }}
}
