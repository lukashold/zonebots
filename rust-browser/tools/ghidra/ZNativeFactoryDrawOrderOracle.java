//Actual40ad60→40b2a0/40c490, including40a790/40a6d0/40ab90 and40a8a0.
//Capacity, spawn/CPU selection, raster calls and global UNIT recounts are
//explicit boundaries.40b180 then executes and must not change producer work.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;
import java.io.*;import java.nio.file.*;
public class ZNativeFactoryDrawOrderOracle extends GhidraScript{
 static final long B=0x5c5e38L,CP=0x5c3f0cL,OWNER=0x72000100L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long pc)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);if(!e.run(toAddr(pc),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat")),weights=Files.readAllBytes(Paths.get("C:/GOG Games/Z/mult.dat"));try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(0x4a8980),weights);for(long a:new long[]{RET,0x45ccb0,0x40c080,0x40f580,0x40c180,0x40c340,0x416c60,0x4189e0,0x40ac30,0x467b80,0x41f610})e.setBreakpoint(toAddr(a));put(e,0x622cb8,2,5000);put(e,0x622cba,2,5000);put(e,0x498c24,2,320);put(e,0x498c20,2,200);put(e,0x622cc0,4,0);put(e,0x4d02db,1,0);put(e,0x4d02d3,1,0);put(e,0x57c94b,1,0);put(e,0x622dba,2,12);put(e,0x622dc4,1,0);put(e,0x622db2,1,0);e.writeMemory(toAddr(0x4cfe38),new byte[16]);e.writeMemory(toAddr(0x622e18),new byte[]{-1,-1,-1,-1});
  for(int s=0;s<2048;s++){int kind=s%2==0?2:4,hold=new int[]{0,1,2,68}[s/2%4],hp=new int[]{0,1,20,100}[s/8%4],owned=s/32%9,owner=new int[]{0,1,4}[s/288%3],blocked=s/864%2,phase=s%4,rate=(hp*(1+owned*100/8/4))/100,remaining=new int[]{-1,0,1,rate,rate+1,1000}[s*7%6],animation=new int[]{0,17,-27}[s%3];
   e.writeMemory(toAddr(B),new byte[35*88]);for(int k=0;k<35;k++)put(e,B+k*88+4,1,255);e.writeMemory(toAddr(CP),new byte[64*30]);for(int k=0;k<64;k++){put(e,CP+k*30,2,k<8?0:-1);put(e,CP+k*30+8,1,k<owned?owner:4);}put(e,OWNER+8,1,owner);put(e,B,2,100);put(e,B+2,2,100);put(e,B+4,1,kind);put(e,B+6,1,1);put(e,B+7,1,0);put(e,B+10,2,animation);put(e,B+28,2,100);put(e,B+30,2,hp);put(e,B+32,4,OWNER);put(e,B+36,4,remaining);put(e,B+40,4,4000+s);put(e,B+44,4,0);put(e,B+54,2,hold);put(e,B+76,4,-1);put(e,0x622d40,2,phase);for(long p:new long[]{0x622d64,0x622d68,0x622d6c,0x622d70}){put(e,p,2,hp);put(e,p+2,2,100);}put(e,0x584972,1,0);
   int clocks=0,clockRemaining=0,clockHold=0,seconds=0,spawns=0;invoke(e,0x40ad60);
   for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){if(budget>30)throw new Exception("unexpected factory calls");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(pc==0x40c080||pc==0x40f580)e.writeRegister("EAX",blocked);else if(pc==0x416c60||pc==0x4189e0)e.writeRegister("EAX",0);else if(pc==0x40c180||pc==0x40c340)spawns++;else if(pc==0x40ac30||pc==0x467b80){clocks++;clockRemaining=(int)read(e,B+36,4);clockHold=(short)read(e,B+54,2);seconds=pc==0x40ac30?(int)read(e,sp+8,4):-1;}else if(pc!=0x45ccb0&&pc!=0x41f610)throw new Exception("unexpected "+Long.toHexString(pc));long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
   int endRemaining=(int)read(e,B+36,4),endTotal=(int)read(e,B+40,4),endHold=(short)read(e,B+54,2),endAnimation=(short)read(e,B+10,2);invoke(e,0x40b180);if(e.readRegister("EIP").longValue()!=RET)throw new Exception("40b180 called anything");if((int)read(e,B+36,4)!=endRemaining||(short)read(e,B+54,2)!=endHold)throw new Exception("40b180 unexpectedly changed work");
   out.println(s+","+kind+","+owner+","+hp+","+owned+","+blocked+","+hold+","+remaining+","+animation+","+phase+","+rate+","+clocks+","+clockRemaining+","+clockHold+","+seconds+","+spawns+","+endRemaining+","+endTotal+","+endHold+","+endAnimation+","+read(e,B+7,1));
  }
 }finally{e.dispose();}}}
}
