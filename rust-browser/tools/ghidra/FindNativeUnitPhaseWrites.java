// Locate native UNIT phase setters for the private lifetime audit.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import java.io.PrintWriter;
public class FindNativeUnitPhaseWrites extends GhidraScript{
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){InstructionIterator it=currentProgram.getListing().getInstructions(true);while(it.hasNext()){Instruction i=it.next();String text=i.toString();if(i.getMnemonicString().equals("MOV")&&i.getDefaultOperandRepresentation(0).contains(" + 0x35]")){Function f=currentProgram.getFunctionManager().getFunctionContaining(i.getAddress());out.println(i.getAddress()+" "+text+" FUNCTION "+(f==null?"null":f.getEntryPoint()+" "+f.getName()));}}}}
}
