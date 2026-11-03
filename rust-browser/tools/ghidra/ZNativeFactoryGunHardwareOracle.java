// Actual17610 creation with available hardware pool slots; captureUNIT+8/6.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
public class ZNativeFactoryGunHardwareOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,B=0x5c3f0cL,H=0x59e6c2L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat"));try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<16;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(R),new byte[240*72]);e.writeMemory(toAddr(B),new byte[64*30]);e.writeMemory(toAddr(0x586880),new byte[240]);e.writeMemory(toAddr(H),new byte[240*44]);for(int i=0;i<240;i++){put(e,H+i*44,2,-1);for(int role=0;role<21;role++)put(e,R+i*72+19+role,1,255);}for(int i=0;i<64;i++)put(e,B+i*30,2,-1);
   put(e,H+14,2,-14000+s);put(e,H+16,2,15000-s);put(e,H+18,1,69);put(e,H+19,1,-90);put(e,H+20,1,-80);put(e,H+23,1,159);
   int product=new int[]{0,8,10,14}[s%4];put(e,0x5ba0a4,1,product);put(e,0x622cc0,4,0);put(e,0x622dc2,1,0);put(e,0x622dc4,1,0);put(e,0x622d1a,2,0);e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",100+s*17);e.writeRegister("EDX",200+s*13);if(!e.run(toAddr(0x417610),null,monitor))throw new Exception(e.getLastError());long unit=read(e,0x4a8bf4,4),hw=read(e,unit+8,4);
   out.println("{\"scenario\":"+s+",\"product\":"+product+",\"hardware\":"+hw+",\"aim6\":"+read(e,unit+6,1)+",\"targetX\":"+(short)read(e,H+14,2)+",\"targetY\":"+(short)read(e,H+16,2)+",\"selected12\":"+read(e,H+18,1)+",\"counter13\":"+read(e,H+19,1)+",\"counter14\":"+read(e,H+20,1)+",\"field17\":"+read(e,H+23,1)+"}");
  }finally{e.dispose();}
 }}}
}
