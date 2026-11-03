//Real56840 BUILD branch with448750,448700,4485a0,44a660,41c290 and
//bridge/ruin/object queries. libm sqrt is isolated;464a20's real zero-length
//and zero-speed guards run before its downstream allocator is isolated.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;import java.math.BigInteger;import java.nio.file.*;import java.util.*;
public class ZNativeBuildFlameOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,O=0x612f80L,B=0x5c5e38L,R=0x6151e0L,G=0x60ec00L,T=0x1300000L,W=T+2048,M=T+4096;
 long[]shot;int emitted,amplitude;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void back(EmulatorHelper e)throws Exception{long sp=e.readRegister("ESP").longValue();e.writeRegister("EIP",read(e,sp,4));e.writeRegister("ESP",sp+4);}
 void call(EmulatorHelper e)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EIP",0x456840);e.getEmulator().setHalt(false);long emitterReturn=0,emitterStack=0;for(int n=0;n<600000;n++){long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(pc==RET)return;if(pc==0x473264){double x=Double.longBitsToDouble(e.readRegister("XMM0").longValue());e.writeRegister("XMM0",BigInteger.valueOf(Double.doubleToRawLongBits(Math.sqrt(x))));back(e);continue;}if(pc==0x44ab00)amplitude=(short)read(e,sp+16,4);if(pc==0x464a20){shot=new long[5];for(int i=0;i<5;i++)shot[i]=read(e,sp+4+i*4,4);emitterReturn=read(e,sp,4);emitterStack=sp;}if(pc==0x464a4b){emitted=1;e.writeRegister("EIP",0x464a3b);continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{String base=System.getenv("Z_GAME_DIR");if(base==null)base="C:/GOG Games/Z";byte[]trig=Files.readAllBytes(Paths.get(base,"sincos.dat"));try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{for(int i=0;i<640;i++)put(e,T+i*2,2,((short)(((trig[i*2]&255)<<8)|(trig[i*2+1]&255)))>>5);put(e,0x586a80,4,T);
 for(int mode=0;mode<14;mode++)for(int flags:new int[]{0,32,128,160})for(int aim:new int[]{0,60,120,255})for(int mp=0;mp<2;mp++)for(int speed:new int[]{0,13}){
  int x=500+mode%4,y=600+mode/4;byte[]map=new byte[16384];if(mode==5||mode==7||mode==11)Arrays.fill(map,(byte)8);if(mode==6)map[((y>>4)*128)+(x>>4)+1]=8;e.writeMemory(toAddr(0x58aaaa),map);e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(W),new byte[44]);put(e,U+8,4,W);put(e,W+5,1,19);put(e,U+0x74,1,15);put(e,U+0x2c,2,300);put(e,U+0x2e,2,400);put(e,U+2,1,aim);put(e,U+0x39,1,mode%8);put(e,U+0x35,1,65);put(e,U+0x46,2,123);put(e,U+0x50,4,3456);put(e,U+0x54,2,x);put(e,U+0x56,2,y);put(e,U+0x69,1,flags);put(e,U+0x79,1,speed);put(e,U+0x7c,1,15);put(e,U+0x30,1,4);put(e,U+0x99,1,mode%2);for(int j=0;j<21;j++)put(e,G+19+j,1,255);put(e,G+19,1,0);
  e.writeMemory(toAddr(O),new byte[200*44]);for(int i=0;i<200;i++)put(e,O+i*44,2,-1);e.writeMemory(toAddr(B),new byte[35*88]);for(int i=0;i<35;i++)put(e,B+i*88+4,1,-1);e.writeMemory(toAddr(R),new byte[12*32]);for(int i=0;i<12;i++)put(e,R+i*32,2,-1);
  if(mode==1||mode==2||mode==10||mode==13){put(e,R,2,x-8);put(e,R+2,2,y-8);put(e,R+4,2,32);put(e,R+6,2,32);put(e,R+15,1,mode==2||mode==13?2:0);if(mode==10){put(e,R+32,2,x-8);put(e,R+34,2,y-8);put(e,R+36,2,32);put(e,R+38,2,32);put(e,R+47,1,2);}}
  if(mode==3||mode==4||mode==11||mode==12){put(e,B,2,x-8);put(e,B+2,2,y-8);put(e,B+4,1,4);put(e,B+0x1e,2,mode==4||mode==12?0:100);}
  if(mode==7||mode==11){put(e,O,2,x-4);put(e,O+2,2,y-4);put(e,O+4,4,0x498f38+(71+mode%7)*26);}if(mode==8){put(e,O,2,x-8);put(e,O+2,2,y-8);put(e,O+4,4,M);put(e,M+18,1,17);}
  put(e,0x622dc4,1,mp);for(int j=0;j<5;j++)put(e,0x622cdc+j*4,4,j==0?mode+1:j+1);shot=null;emitted=0;amplitude=-999;call(e);StringBuilder row=new StringBuilder(mode+","+flags+","+aim+","+mp+","+speed+","+(shot==null?0:1)+","+emitted+","+amplitude);for(int i=0;i<5;i++)row.append(',').append(shot==null?-1:shot[i]);row.append(',').append(read(e,U+0x74,1)).append(',').append(read(e,W+5,1)).append(',').append((short)read(e,U+0x46,2)).append(',').append(read(e,U+0x39,1));for(int j=0;j<5;j++)row.append(',').append(read(e,0x622cdc+j*4,4));out.println(row);out.flush();
 } }finally{e.dispose();}}}
}
