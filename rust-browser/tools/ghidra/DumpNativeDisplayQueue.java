import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.Instruction;
import java.io.PrintWriter;
public class DumpNativeDisplayQueue extends GhidraScript{public void run()throws Exception{try(PrintWriter o=new PrintWriter(getScriptArgs()[0])){Instruction i=currentProgram.getListing().getInstructionAt(toAddr(0x435b90));if(i==null)i=currentProgram.getListing().getInstructionAfter(toAddr(0x435b90));while(i!=null&&i.getAddress().compareTo(toAddr(0x436150))<0){o.println(i.getAddress()+" "+i);i=i.getNext();}}}}
