//405040→4549f0→54630 draw dispatch, isolating the final graphics submission.
//Coordinates, palette, sprite, signed Y offset, size and angle are native.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeAmbientDrawOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,B=0x60b028L,D=0x62dc50L;String draw;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f,long cx,long dx)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<50000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;if(pc==0x465cb0||pc==0x46bde0||pc==0x466090||pc==0x466180||pc==0x466ac0){long sp=e.readRegister("ESP").longValue();if(pc==0x466090||pc==0x466180||pc==0x466ac0){boolean quad=pc==0x466ac0;draw=(quad?2:1)+","+(short)e.readRegister("ECX").intValue()+","+(short)e.readRegister("EDX").intValue()+","+(short)read(e,sp+(quad?4:8),2)+","+read(e,0x477063,1)+","+(quad?(short)read(e,sp+8,2):0)+","+(quad?(short)read(e,sp+12,2):0);}e.writeRegister("ESP",sp+4);e.writeRegister("EIP",read(e,sp,4));continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int flags52:new int[]{0,1,2,3,4,8,12,68})for(int flags53:new int[]{0,16,32,64})for(int frame:new int[]{-3,0,2})for(int offset:new int[]{0,6,16,255})for(int angle:new int[]{0,127,128,255}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(B),new byte[62]);e.writeMemory(toAddr(D),new byte[16]);e.writeMemory(toAddr(0x628e18),new byte[16]);put(e,B,2,500);put(e,B+2,2,600);for(int[]p:new int[][]{{20,100},{22,200},{24,300},{28,400}})put(e,B+p[0],2,p[1]);put(e,B+44,1,96);put(e,B+45,1,frame);put(e,B+50,1,angle);put(e,B+51,1,offset);put(e,B+52,1,flags52);put(e,B+53,1,flags53);put(e,0x622cb8,4,0);put(e,0x498c24,2,1024);put(e,0x498c20,2,1024);put(e,0x62d5d8,4,0x628e18);put(e,0x680cfc,4,0);put(e,0x4d02df,1,0);put(e,0x477063,1,0);put(e,0x47717f,1,0);draw="NONE";call(e,0x405040,B,D);put(e,0x62b3a4,4,e.readRegister("EAX").longValue());call(e,0x4549f0,0,0);if(draw.equals("NONE"))throw new Exception("Missing native draw");out.println(flags52+","+flags53+","+frame+","+offset+","+angle+","+draw);out.flush();
   }finally{e.dispose();}
  }
 }}
}
