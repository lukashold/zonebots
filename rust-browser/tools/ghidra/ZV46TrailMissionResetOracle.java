//Actual33ca0 initialization prefix4340c0..43416b. No stack/gameplay input
//is synthesized. Only all64x24 warm job bytes and adjacent guard bytes vary.
//CPU hot EAX is cleared by actualXOR4340c0; actualREP count is384dwords.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.*;
public class ZV46TrailMissionResetOracle extends GhidraScript{
 static final long J=0x62d5f8L;
 String hex(byte[]b){StringBuilder s=new StringBuilder();for(byte n:b)s.append(String.format("%02x",n&255));return s.toString();}
 public void run()throws Exception{try(PrintWriter o=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<256;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{byte[]raw=new byte[1536+32];for(int i=0;i<raw.length;i++)raw[i]=(byte)(s*37+i*23+(i/24)*11+17);e.writeMemory(toAddr(J-16),raw);e.writeRegister("EAX",0x13572468L+s);e.writeRegister("ECX",0x12345678);e.writeRegister("EDI",0x23456789);e.writeRegister("EFLAGS",0x202);e.setBreakpoint(toAddr(0x43416b));if(!e.run(toAddr(0x4340c0),null,monitor))throw new Exception(e.getLastError());
  o.println("{\"scenario\":"+s+",\"entry\":4407488,\"stop\":"+e.readRegister("EIP").longValue()+",\"before\":\""+hex(raw)+"\",\"after\":\""+hex(e.readMemory(toAddr(J-16),raw.length))+"\",\"eax\":"+e.readRegister("EAX").longValue()+",\"ecx\":"+e.readRegister("ECX").longValue()+",\"edi\":"+e.readRegister("EDI").longValue()+"}");o.flush();
 }finally{e.dispose();}}}}
}
