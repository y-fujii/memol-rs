#include "../imgui/imgui.cpp"
#include "../imgui/imgui_draw.cpp"


void wrapGetWindowPos( ImVec2& out ) {
	out = ImGui::GetWindowPos();
}

void wrapGetWindowSize( ImVec2& out ) {
	out = ImGui::GetWindowSize();
}

void wrapGetMouseDragDelta( ImVec2& out, int b, float t ) {
	out = ImGui::GetMouseDragDelta( b, t );
}
