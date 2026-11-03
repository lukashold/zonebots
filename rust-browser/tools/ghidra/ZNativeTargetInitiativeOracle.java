// Complete directed 100-record target scan44d140/44cf80 including cumulative buckets,
// native filtering and terrain LOS. The imported CRT sqrt uses host arithmetic.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.math.BigInteger;
public class ZNativeTargetInitiativeOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,B=0x60b028L,D=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<300000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;if(pc==0x47326a){long sp=e.readRegister("ESP").longValue(),dst=read(e,sp+4,4),src=read(e,sp+8,4),len=read(e,sp+12,4);if(len>4096)throw new Exception("Invalid memcpy size");e.writeMemory(toAddr(dst),e.readMemory(toAddr(src),(int)len));e.writeRegister("EAX",dst);e.writeRegister("ESP",sp+4);e.writeRegister("EIP",read(e,sp,4));continue;}if(pc==0x473264){double x=Double.longBitsToDouble(e.readRegister("XMM0").longValue());e.writeRegister("XMM0",BigInteger.valueOf(Double.doubleToRawLongBits(Math.sqrt(x))));long sp=e.readRegister("ESP").longValue(),back=read(e,sp,4);e.writeRegister("ESP",sp+4);e.writeRegister("EIP",back);continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int direction=0;direction<8;direction++)for(int fixture=0;fixture<6;fixture++)for(int blocked=0;blocked<2;blocked++)for(int gate=0;gate<3;gate++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[0xc0]);e.writeMemory(toAddr(B),new byte[100*62]);put(e,0x58687b,1,0);put(e,U+0x2c,2,500);put(e,U+0x2e,2,600);put(e,U+0x10,4,4895);put(e,U+0x38,1,(600>>7)*16+(500>>7));put(e,U+0x39,1,direction);put(e,U+0x8a,1,gate==1?16:0);put(e,U+0x64,1,gate==2?32:0);byte[]map=new byte[16384];java.util.Arrays.fill(map,(byte)10);if(blocked==1)for(int y=0;y<128;y++)map[y*128+34]=11;e.writeMemory(toAddr(0x58aaaa),map);put(e,0x4948d4,4,0x499ab0);
    for(int i=0;i<100;i++){put(e,B+i*62,2,-1);if(i>=64||i%11==fixture)continue;int x=500+(i*37+fixture*13)%705-352,y=600+(i*71+fixture*17)%705-352;put(e,B+i*62,2,x);put(e,B+i*62+2,2,y);put(e,B+i*62+42,2,(y>>7)*16+(x>>7));put(e,B+i*62+52,1,i%17==0?2:0);put(e,B+i*62+53,1,i%13==0?16:0);put(e,B+i*62+60,1,i%19==0?4:0);}
    call(e,0x4064f0);put(e,D,1,55);put(e,D+1,1,77);put(e,S+4,4,D);put(e,S+8,4,D+1);call(e,0x44d140);out.println(direction+","+fixture+","+blocked+","+gate+","+read(e,D,1)+","+read(e,D+1,1));out.flush();
   }finally{e.dispose();}
  }
 }}
}
