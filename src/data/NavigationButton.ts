import {House, BookOpen, Layers, LayoutDashboard, Settings2 } from 'lucide-react'
import { NavigationButtonType } from '../types/NavigationButton'

export const buttons: NavigationButtonType[] = [
    {icon: House, labelKey: 'nav.home', to: '/'},
    {icon: BookOpen, labelKey: 'nav.review', to: '/review'},
    {icon: Layers, labelKey: 'nav.decks', to: '/decks'},
    {icon: LayoutDashboard, labelKey: 'nav.dashboard', to: '/dashboard'},
    {icon: Settings2, labelKey: 'nav.settings', to: '/settings'},
]
