//Executes the actual435c12..435e21 Main segment and actual5a240 fixed-slot
//loop. Callee bodies, including rasterizers and UNIT handlers, are explicit
//boundaries; this proves producer/callback ordering, not complete Main parity.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import ghidra.program.model.listing.Instruction;
import java.io.PrintWriter;
import java.util.*;
public class ZNativeMainDisplayOracle extends GhidraScript{
 static final long S=0x7fff0000L,END=0x435e21L,U=0x5e30c8L,O=0x612f80L,A=0x60b028L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  Set<Long>stubs=new TreeSet<>();Instruction i=currentProgram.getListing().getInstructionAt(toAddr(0x435c12));while(i!=null&&i.getAddress().getOffset()<END){if(i.getMnemonicString().equals("CALL"))for(ghidra.program.model.address.Address a:i.getFlows())stubs.add(a.getOffset());i=i.getNext();}
  for(long a:new long[]{0x456a90,0x43e890,0x4064f0,0x45a9f0,0x45ccb0,0x40b180,0x44b280,0x45a080})stubs.add(a);for(long a:stubs)e.setBreakpoint(toAddr(a));e.setBreakpoint(toAddr(END));
  for(int scenario=0;scenario<96;scenario++){
   e.setBreakpoint(toAddr(0x45a240));e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(O),new byte[200*44]);e.writeMemory(toAddr(A),new byte[100*62]);
   int mask=scenario%16,mode=new int[]{0,1,16}[scenario/16%3],variation=scenario/48;int[]slots={0,7,31,239};for(int k=0;k<4;k++)if((mask&(1<<k))!=0){put(e,U+slots[k]*192+49,1,4);put(e,U+slots[k]*192+50,2,255);}
   for(int k=0;k<200;k++)put(e,O+k*44,2,-1);for(int k=0;k<100;k++)put(e,A+k*62,2,-1);for(int k=0;k<8;k++){put(e,O+k*44,2,(k+scenario)%3-1);put(e,A+k*62,2,(k+scenario+variation)%3-1);}
   put(e,0x622cc0,4,mode);put(e,0x5b6d9c,4,0);put(e,0x62d5d2,1,0);put(e,0x622e14,1,0);put(e,0x622e15,1,0);put(e,0x62b3a4,4,0x62dc50);e.writeRegister("ESP",S);e.writeRegister("ECX",0);ArrayList<Long>trace=new ArrayList<>();
   if(!e.run(toAddr(0x435c12),null,monitor))throw new Exception(e.getLastError());
   for(int budget=0;e.readRegister("EIP").longValue()!=END;budget++){
    if(budget>500)throw new Exception("Main boundary overflow");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(!stubs.contains(pc))throw new Exception("unexpected "+Long.toHexString(pc));long token=pc;
    if(pc==0x45a080)token|=((e.readRegister("ECX").longValue()-U)/192)<<24;
    if(pc==0x43f260)token|=((e.readRegister("ECX").longValue()-O)/44)<<24;
    if(pc==0x405820)token|=((e.readRegister("ECX").longValue()-A)/62)<<24;
    trace.add(token);
    if(pc==0x45a240){e.clearBreakpoint(toAddr(pc));if(!e.run(toAddr(pc),null,monitor))throw new Exception(e.getLastError());continue;}
    if(pc==0x43f260||pc==0x405820)e.writeRegister("EAX",e.readRegister("EDX"));long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
   }
   int h=0x811c9dc5;StringJoiner seq=new StringJoiner(";");for(long token:trace){h=(h^(int)token)*16777619;seq.add(Long.toUnsignedString(token));}out.println(scenario+","+mode+","+mask+","+trace.size()+","+Integer.toUnsignedString(h)+","+seq);
  }
 }finally{e.dispose();}}}
}
