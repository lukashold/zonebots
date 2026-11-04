import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.*;import java.nio.file.*;
// Numeric synthetic fixtures: actual40c080 including10510/10570 and f580; Grunt is profile0,
// so this wrapper needs no intercepted sector-gun-count leaf.
public class ZRound37ProductionCapacityOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,F=0x72000000L,CP=0x5c3f0cL,U=0x5e30c8L,BUILD=0x5c5e38L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat"));try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<512;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
 int rawLimit=new int[]{50,120,127,128,255,0,1}[(s/11)%7],units=new int[]{48,49,50,51,119,120,127,-128,-1,0,1}[s%11],queued=(s/77)%5,pending=(s/385)%2,free=new int[]{220,7,6}[(s/33)%3];
 e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(F),new byte[88]);e.writeMemory(toAddr(CP),new byte[30]);e.writeMemory(toAddr(BUILD),new byte[35*88]);e.writeMemory(toAddr(U),new byte[240*192]);put(e,F+32,4,CP);put(e,F+7,1,2);put(e,CP+8,1,0);
 for(int i=0;i<35;i++)put(e,BUILD+i*88+4,1,255);put(e,BUILD+4,1,4);put(e,BUILD+30,2,100);put(e,BUILD+32,4,CP);for(int i=0;i<4;i++)put(e,BUILD+82+i,1,i<queued?0:255);for(int i=0;i<5;i++)put(e,0x5ba0a4+i,1,255);if(pending!=0)put(e,0x5ba0a4,1,0);put(e,0x622ddd,1,units);put(e,0x5c47b4,1,rawLimit);for(int i=0;i<240;i++)put(e,U+i*192+49,1,i<free?0:4);
 e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",F);e.writeRegister("EDX",2);if(!e.run(toAddr(0x40c080),null,monitor))throw new Exception(e.getLastError());out.println("{\"scenario\":"+s+",\"limit\":"+(byte)rawLimit+",\"units\":"+units+",\"queued\":"+queued+",\"pending\":"+pending+",\"free\":"+free+",\"blocked\":"+(e.readRegister("EAX").intValue()&255)+"}");
 }finally{e.dispose();}}}}
}
