// Numeric execution of43f260's signed OBJECT contact-delay decrement.
import ghidra.app.emulator.EmulatorHelper;
import ghidra.program.model.listing.Instruction;
import java.io.PrintWriter;
public class ZNativeObjectDelayOracle extends ZNativeReactionsOracle{
 public void run()throws Exception{String path=getScriptArgs()[0];try(PrintWriter out=new PrintWriter(path)){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.setBreakpoint(toAddr(0x43f276));for(int value=0;value<256;value++){put(e,U+0x1f,1,value);e.writeRegister("ECX",U);e.writeRegister("EDX",P);e.writeRegister("ESP",S);put(e,S,4,RET);if(!e.run(toAddr(0x43f260),null,monitor))throw new Exception(e.getLastError());out.println("{\"value\":"+value+",\"next\":"+read(e,U+0x1f,1)+"}");}
 }finally{e.dispose();}}
 try(PrintWriter out=new PrintWriter(path+".caller.txt")){Instruction i=currentProgram.getListing().getInstructionAt(toAddr(0x435d50));if(i==null)i=currentProgram.getListing().getInstructionAfter(toAddr(0x435d50));for(;i!=null&&i.getAddress().getOffset()<0x435dc8;i=i.getNext())out.println(i.getAddress()+" "+i);}
 }
}
