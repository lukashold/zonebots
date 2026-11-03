// Full44d3a0/44d290 with native spatial lists and terrain LOS. Only CRT sqrt
//is replaced by its exact host arithmetic, as in the existing scan oracle.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.math.BigInteger;
public class ZNativeObjectInitiativeOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,O=0x612f80L,D=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<300000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;if(pc==0x473264){double x=Double.longBitsToDouble(e.readRegister("XMM0").longValue());e.writeRegister("XMM0",BigInteger.valueOf(Double.doubleToRawLongBits(Math.sqrt(x))));long sp=e.readRegister("ESP").longValue(),back=read(e,sp,4);e.writeRegister("ESP",sp+4);e.writeRegister("EIP",back);continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int direction=0;direction<8;direction++)for(int relative=0;relative<3;relative++)for(int cycle=0;cycle<2;cycle++)for(int blocked=0;blocked<2;blocked++)for(int team=0;team<2;team++)for(int gate=0;gate<3;gate++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[0xc0]);e.writeMemory(toAddr(O),new byte[200*44]);put(e,0x58687b,1,0);put(e,U+0x2c,2,500);put(e,U+0x2e,2,600);put(e,U+0x10,4,((600+8)>>4)*128+((500+8)>>4));put(e,U+0x38,1,(600>>7)*16+(500>>7));put(e,U+0x31,1,team|4);put(e,U+0x39,1,direction);put(e,U+0xa4,1,(direction+3)&7);put(e,U+0x9c,1,0x82);put(e,U+0x71,1,cycle==1?8:0);put(e,U+0x8a,1,gate==1?16:0);put(e,U+0x64,1,gate==2?32:0);byte[]map=new byte[16384];java.util.Arrays.fill(map,(byte)10);if(blocked==1)for(int y=0;y<128;y++)map[y*128+34]=11;e.writeMemory(toAddr(0x58aaaa),map);put(e,0x4948d4,4,0x499ab0);
    int fixture=direction+relative*8;for(int i=0;i<200;i++){put(e,O+i*44,2,-1);if(i>=24||i%7==fixture%7)continue;int x=500+(i*47+fixture*13)%321-160,y=600+(i*71+fixture*17)%321-160;put(e,O+i*44,2,x);put(e,O+i*44+2,2,y);put(e,O+i*44+12,1,(y>>7)*16+(x>>7));put(e,O+i*44+19,1,i%3==0?3:1<<(i&1));}
    call(e,0x43e890);put(e,D,1,55);put(e,D+1,1,77);put(e,S+4,4,D);put(e,S+8,4,D+1);e.writeRegister("EDX",relative);call(e,0x44d3a0);
    out.println(direction+","+relative+","+cycle+","+blocked+","+team+","+gate+","+read(e,D,1)+","+read(e,D+1,1)+","+read(e,U+0x9c,1)+","+read(e,U+0xa4,1));out.flush();
   }finally{e.dispose();}
  }
 }}
}
